use axum::extract::{State, Query, Path};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use crate::server::state::AppState;

pub async fn update_state(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    {
        let mut ui = state.ui_state.write().await;
        if let Some(obj) = payload.as_object() {
            if let Some(ui_obj) = ui.as_object_mut() {
                for (k, v) in obj { ui_obj.insert(k.clone(), v.clone()); }
            }
        }
    }

    let state_file = crate::expand_path("~/.vellum/state.json");
    if let Ok(content) = serde_json::to_string_pretty(&*state.ui_state.read().await) {
        let _ = tokio::fs::write(state_file, content).await;
    }

    Json(json!({"status": "saved"})).into_response()
}

pub async fn trigger_full_reset(State(state): State<Arc<AppState>>) -> Response {
    log::info!("Full library reset triggered");
    {
        let mut lib = state.library.write().await;
        lib.scan().await;
    }
    
    let payload = {
        let lib = state.library.read().await;
        let ui = state.ui_state.read().await;
        json!({
            "type": "INIT",
            "data": lib.albums,
            "ui_state": *ui
        }).to_string()
    };

    let _ = state.tx.send(payload);
    Json(json!({"status": "ok"})).into_response()
}

pub async fn trigger_reload(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    if let Some(path) = params.get("path") {
        let mut lib = state.library.write().await;
        if let Some(updated) = lib.update_album(path) {
            let album_name = updated.album_data.other.get("ALBUM")
                .and_then(|v| v.as_str())
                .unwrap_or(&updated.id);
            
            log::info!("Updated: {}", album_name);

            let _ = state.tx.send(json!({
                "type": "UPDATE",
                "id": updated.id,
                "payload": updated
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
    let path = state.config.library_root.join(id);
    if path.exists() {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        return Json(json!({"status": "ok"})).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}
