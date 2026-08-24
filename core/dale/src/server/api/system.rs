use crate::server::inotify::handler::RecompiledAlbumItem;
use crate::server::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct AlbumIngestPayload {
    pub album_dir: PathBuf,
    pub id: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    pub lock_json: String,
    pub eval_res: Option<serde_json::Value>,
    #[serde(default)]
    pub dependencies: Vec<PathBuf>,
    #[serde(default)]
    pub modified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTriggerPayload {
    pub path: Option<String>,
    #[serde(default)]
    pub force: bool,
    pub jobs: Option<usize>,
    #[serde(default)]
    pub silent: bool,
}

pub async fn trigger_update(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateTriggerPayload>,
) -> Response {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

    let target_path = payload.path.map(PathBuf::from);
    let force = payload.force;
    let jobs = payload.jobs;
    let silent = payload.silent;

    tokio::spawn(async move {
        if let Err(e) = crate::update::run_server_update(
            state,
            target_path,
            force,
            jobs,
            silent,
            Some(tx.clone()),
        )
        .await
        {
            let _ = tx.send(format!("Update failed: {e}")).await;
        }
    });

    let stream = futures::stream::unfold(rx, |mut rx| async move {
        let msg = rx.recv().await?;
        Some((Ok::<_, std::convert::Infallible>(format!("{msg}\n")), rx))
    });

    Response::builder()
        .header(
            axum::http::header::CONTENT_TYPE,
            "text/plain; charset=utf-8",
        )
        .body(axum::body::Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub async fn get_tracked_albums(State(state): State<Arc<AppState>>) -> Response {
    let full_rescan = state
        .full_rescan_needed
        .swap(false, std::sync::atomic::Ordering::Relaxed);
    let tracked: Vec<String> = state
        .tracked_albums
        .lock()
        .map_or_else(|_| Vec::new(), |mut guard| guard.drain().collect());
    Json(json!({
        "full_rescan": full_rescan,
        "tracked_albums": tracked
    }))
    .into_response()
}

pub async fn get_interface_config(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let name = name.replace('-', "_");
    let intf_cfg = {
        let guard = state.config.read().await;
        guard.interfaces.get(&name).cloned()
    };
    if let Some(cfg) = intf_cfg {
        return Json(cfg.config).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn serve_interface_asset(
    Path((name, asset_path)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let name = name.replace('-', "_");
    let (intf_cfg, config_dir) = {
        let guard = state.config.read().await;
        (
            guard.interfaces.get(&name).cloned(),
            guard.config_dir.clone(),
        )
    };

    let Some(cfg) = intf_cfg else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let parts: Vec<&str> = asset_path.splitn(2, '/').collect();
    let key = parts[0];
    let subpath = parts.get(1).copied();

    let Some(asset_val) = cfg.assets.get(key) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let p = libdale::utils::expand_path(asset_val);
    let resolved = if p.is_absolute() {
        p
    } else {
        config_dir.join(p)
    };
    let Ok(resolved_canon) = tokio::fs::canonicalize(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Ok(meta) = tokio::fs::metadata(&resolved_canon).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let target_path = if meta.is_file() {
        if subpath.is_some() {
            return StatusCode::NOT_FOUND.into_response();
        }
        resolved_canon
    } else if meta.is_dir() {
        let Some(sub) = subpath else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let full_path = resolved_canon.join(sub);
        let Ok(full_canon) = tokio::fs::canonicalize(&full_path).await else {
            return StatusCode::FORBIDDEN.into_response();
        };
        if !full_canon.starts_with(&resolved_canon) || !full_canon.is_file() {
            return StatusCode::FORBIDDEN.into_response();
        }
        full_canon
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Ok(mut file) = tokio::fs::File::open(&target_path).await {
        let mut buf = Vec::new();
        if tokio::io::AsyncReadExt::read_to_end(&mut file, &mut buf)
            .await
            .is_ok()
        {
            let mime = match target_path.extension().and_then(|e| e.to_str()) {
                Some("css") => "text/css",
                Some("frag" | "glsl" | "vert") => "text/plain",
                Some("js") => "application/javascript",
                Some("json") => "application/json",
                Some("png") => "image/png",
                Some("jpg" | "jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                Some("woff2") => "font/woff2",
                _ => "application/octet-stream",
            };
            return ([(axum::http::header::CONTENT_TYPE, mime)], buf).into_response();
        }
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn update_state(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let content = {
        let mut ui = state.ui_state.write().await;
        if let Some(obj) = payload.as_object()
            && let Some(ui_obj) = ui.as_object_mut()
        {
            for (k, v) in obj {
                ui_obj.insert(k.clone(), v.clone());
            }
        }
        serde_json::to_string_pretty(&*ui).ok()
    };

    if let Some(data) = content {
        let state_file = {
            let guard = state.config.read().await;
            guard.state_root.join("state.json")
        };
        let _ = tokio::fs::write(state_file, data).await;
    }

    Json(json!({"status": "saved"})).into_response()
}

pub async fn notify_force_update() -> Response {
    log::info!("Force updating library...");
    Json(json!({"status": "ok"})).into_response()
}

pub async fn trigger_full_reset(State(state): State<Arc<AppState>>) -> Response {
    log::info!("Rebuilding library database...");
    let start_time = std::time::Instant::now();

    let music_directory = state.config.read().await.music_directory.clone();
    let logic_arc = Arc::clone(&state.logic);

    let (album_count, manifest) = tokio::task::spawn_blocking(move || {
        let scanner = crate::server::library::scanner::Library::new(music_directory);
        let mut logic = logic_arc.blocking_write();
        scanner.scan(&mut logic);
        (logic.dict.len(), logic.manifest.clone())
    })
    .await
    .unwrap_or_else(|_| (0, libdale::lua::LogicManifest::default()));

    let elapsed = start_time.elapsed().as_millis();
    log::info!("Updated {album_count} albums.");
    log::info!("Rebuilt Logic Engine in {elapsed}ms.");

    let _ = state.tx.send(
        json!({
            "type": "LOGIC_UPDATE",
            "manifest": manifest
        })
        .to_string(),
    );
    Json(json!({"status": "ok"})).into_response()
}

pub async fn trigger_reload(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    if let Some(path) = params.get("path") {
        let target_path = PathBuf::from(path);
        tokio::spawn(async move {
            if let Err(e) = crate::update::run_server_update(
                state,
                Some(target_path),
                true,
                None,
                true,
                None,
            )
            .await
            {
                log::error!("Failed to reload album: {e}");
            }
        });
        return Json(json!({"status": "ok"})).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn ingest_reload_payloads(
    State(state): State<Arc<AppState>>,
    Json(payloads): Json<Vec<AlbumIngestPayload>>,
) -> Response {
    let recompiled_items: Vec<RecompiledAlbumItem> = payloads
        .into_iter()
        .map(|p| {
            (
                p.album_dir,
                p.id,
                p.lock_json,
                p.eval_res,
                p.dependencies.into_iter().collect(),
            )
        })
        .collect();

    crate::server::inotify::handler::ingest_and_broadcast_albums(
        std::collections::HashSet::new(),
        recompiled_items,
        false,
        &state,
    )
    .await;
    StatusCode::OK.into_response()
}

pub async fn force_update_album(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path_opt = {
        let logic = state.logic.read().await;
        logic.path_by_id.get(&id).cloned()
    };

    if let Some(path) = path_opt {
        tokio::spawn(async move {
            if let Err(e) = crate::update::run_server_update(
                state,
                Some(path),
                true,
                None,
                true,
                None,
            )
            .await
            {
                log::error!("Failed to force update album: {e}");
            }
        });
        return Json(json!({"status": "ok"})).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn run_query(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let q_str = payload
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let logic = state.logic.read().await;
    let ids = logic.find_ids(q_str);
    let mut results = Vec::new();
    for id in ids {
        if let Some(path) = logic.path_by_id.get(&id)
            && let Some(json_str) = logic.get_album_json(&id)
            && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&json_str)
        {
            results.push(json!({
                "id": id,
                "path": path,
                "lock": lock_json,
            }));
        }
    }
    Json(results).into_response()
}
