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
        let state_file = crate::expand_path("~/.vellum/state.json");
        let _ = tokio::fs::write(state_file, data).await;
    }

    Json(json!({"status": "saved"})).into_response()
}

pub async fn trigger_full_reset(State(state): State<Arc<AppState>>) -> Response {
    log::info!("Full library reset triggered");
    {
        let config_guard = state.config.read().await;
        let mut query = state.query.lock().await;
        let scanner = crate::server::library::scanner::Library::new(config_guard.library_root.clone());
        scanner.scan(&mut query);
    }
    
    let _ = state.tx.send(json!({"type": "LOGIC_UPDATE"}).to_string());
    Json(json!({"status": "ok"})).into_response()
}

pub async fn trigger_reload(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    if let Some(path) = params.get("path") {
        let config_guard = state.config.read().await;
        let (internal_id, dict_entry) = {
            let mut query = state.query.lock().await;
            let scanner = crate::server::library::scanner::Library::new(config_guard.library_root.clone());
            let id = scanner.update_album(path, &mut query);
            let entry = id.as_ref().and_then(|i| query.dict.get(i).cloned());
            (id, entry)
        };

        if let Some(id) = internal_id {
            log::info!("Updated: {}", id);
            
            let _ = state.tx.send(json!({
                "type": "ALBUM_UPDATED",
                "id": id,
                "dictEntry": dict_entry.unwrap_or(json!({}))
            }).to_string());
            
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
            .arg(path)
            .spawn();
        return Json(json!({"status": "ok"})).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}
