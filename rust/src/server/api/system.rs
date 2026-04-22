use crate::server::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::sync::Arc;

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
    let album_count = {
        let config_guard = state.config.read().await;
        let mut query = state.query.lock().await;
        let scanner = crate::server::library::scanner::Library::new(config_guard.library_root.clone());
        scanner.scan(&mut query);
        query.dict.len()
    };
    
    let elapsed = start_time.elapsed().as_millis();
    log::info!("Updated {} albums.", album_count);
    log::info!("Rebuilt Query Engine in {}ms.", elapsed);
    
    let _ = state.tx.send(json!({"type": "LOGIC_UPDATE"}).to_string());
    Json(json!({"status": "ok"})).into_response()
}

pub async fn trigger_batch_reload(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    Json(paths): Json<Vec<String>>,
) -> Response {
    let start_time = std::time::Instant::now();
    let compile_time = params.get("time").map(|s| s.as_str()).unwrap_or("0");
    let config_guard = state.config.read().await;
    let mut processed_ids = Vec::new();
    let mut removed_ids = Vec::new();

    {
        let mut query = state.query.lock().await;
        let scanner = crate::server::library::scanner::Library::new(config_guard.library_root.clone());
        for path in &paths {
            if let Some(res) = scanner.update_album(path, &mut query) {
                match res {
                    crate::server::library::scanner::UpdateResult::Updated(id) => processed_ids.push(id),
                    crate::server::library::scanner::UpdateResult::Removed(id) => removed_ids.push(id),
                }
            }
        }
        if !processed_ids.is_empty() || !removed_ids.is_empty() {
            let _ = query.build_cache();
        }
    }

    if !processed_ids.is_empty() || !removed_ids.is_empty() {
        let elapsed = start_time.elapsed().as_millis();
        log::info!("Updated {} albums, Removed {} albums in {}ms.", processed_ids.len(), removed_ids.len(), compile_time);
        log::info!("Rebuilt Query Engine in {}ms.", elapsed);
        
        if processed_ids.len() == 1 && removed_ids.is_empty() {
            let dict_entry = {
                let query = state.query.lock().await;
                query.dict.get(&processed_ids[0]).cloned()
            };
            let _ = state.tx.send(json!({
                "type": "ALBUM_UPDATED",
                "id": processed_ids[0],
                "dictEntry": dict_entry.unwrap_or(json!({}))
            }).to_string());
        } else if removed_ids.len() == 1 && processed_ids.is_empty() {
            let _ = state.tx.send(json!({
                "type": "ALBUM_REMOVED",
                "id": removed_ids[0]
            }).to_string());
        } else {
            let _ = state.tx.send(json!({"type": "LOGIC_UPDATE"}).to_string());
        }
    }

    Json(processed_ids).into_response()
}

pub async fn trigger_reload(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let start_time = std::time::Instant::now();
    if let Some(path) = params.get("path") {
        let config_guard = state.config.read().await;
        let (update_res, dict_entry) = {
            let mut query = state.query.lock().await;
            let scanner = crate::server::library::scanner::Library::new(config_guard.library_root.clone());
            let res = scanner.update_album(path, &mut query);
            if res.is_some() {
                let _ = query.build_cache();
            }
            let entry = match &res {
                Some(crate::server::library::scanner::UpdateResult::Updated(id)) => query.dict.get(id).cloned(),
                _ => None,
            };
            (res, entry)
        };

        if let Some(res) = update_res {
            let elapsed = start_time.elapsed().as_millis();
            log::info!("Processed 1 album.");
            log::info!("Rebuilt Query Engine in {}ms.", elapsed);
            
            match res {
                crate::server::library::scanner::UpdateResult::Updated(id) => {
                    let _ = state.tx.send(json!({
                        "type": "ALBUM_UPDATED",
                        "id": id,
                        "dictEntry": dict_entry.unwrap_or(json!({}))
                    }).to_string());
                }
                crate::server::library::scanner::UpdateResult::Removed(id) => {
                    let _ = state.tx.send(json!({
                        "type": "ALBUM_REMOVED",
                        "id": id
                    }).to_string());
                }
            }
            
            return Json(json!({"status": "ok"})).into_response();
        }
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn open_album_folder(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path = {
        let config_guard = state.config.read().await;
        config_guard.library_root.join(id)
    };
    if path.exists() {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        return Json(json!({"status": "ok"})).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn open_lock_file(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path = {
        let config_guard = state.config.read().await;
        config_guard.library_root.join(id).join("metadata.lock.json")
    };
    if path.exists() {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        return Json(json!({"status": "ok"})).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn open_manifest_file(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path = {
        let config_guard = state.config.read().await;
        config_guard.library_root.join(id).join("metadata.toml")
    };
    if path.exists() {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        return Json(json!({"status": "ok"})).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn force_update_album(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path = {
        let config_guard = state.config.read().await;
        config_guard.library_root.join(id)
    };
    if path.exists() {
        let _ = std::process::Command::new("vellum")
            .arg("update")
            .arg("--force")
            .arg("--silent")
            .arg(path)
            .spawn();
        return Json(json!({"status": "ok"})).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn run_query(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if let Some(query_str) = payload.get("query").and_then(|q| q.as_str()) {
        let expanded = crate::server::query::expand_shorthand(query_str);
        let query = state.query.lock().await;
        match query.query_ids(&expanded) {
            Ok(ids) => return Json(ids).into_response(),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        }
    }
    StatusCode::BAD_REQUEST.into_response()
}
