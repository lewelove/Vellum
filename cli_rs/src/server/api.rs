use axum::extract::ws as ax_ws;
use axum::{
    extract::{Path, State, WebSocketUpgrade, Query},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use axum::http::{StatusCode, header};
use tokio::fs::File;

use crate::server::AppState;
use crate::server::mpd_engine::MpdCommand;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: ax_ws::WebSocket, state: Arc<AppState>) {
    let init_payload = {
        let lib = state.library.read().await;
        let ui = state.ui_state.read().await;
        json!({
            "type": "INIT",
            "data": lib.albums,
            "ui_state": *ui
        }).to_string()
    };
    
    if socket.send(ax_ws::Message::Text(init_payload.into())).await.is_err() {
        return;
    }

    // Trigger an immediate status refresh so the UI receives MPD_STATUS right after INIT
    state.mpd_engine.send(MpdCommand::Refresh).await;

    let mut rx = state.tx.subscribe();
    loop {
        tokio::select! {
            Some(msg) = socket.recv() => {
                if let Ok(ax_ws::Message::Close(_)) | Err(_) = msg {
                    break;
                }
            }
            Ok(msg) = rx.recv() => {
                if socket.send(ax_ws::Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

pub async fn update_state(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    {
        let mut ui = state.ui_state.write().await;
        if let Some(obj) = payload.as_object() {
            if let Some(ui_obj) = ui.as_object_mut() {
                for (k, v) in obj {
                    ui_obj.insert(k.clone(), v.clone());
                }
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

pub async fn get_cover_thumbnail(
    Path(hash): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if let Some(root) = &state.config.thumbnail_root {
        let path = root.join(format!("{}.png", hash));
        return serve_image(path).await;
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn get_album_cover(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path_opt = {
        let lib = state.library.read().await;
        lib.album_map.get(&id).and_then(|a| {
            a.album_data.cover_path.as_ref().map(|cp| {
                state.config.library_root.join(&id).join(cp)
            })
        })
    };

    if let Some(path) = path_opt {
        return serve_image(path).await;
    }
    StatusCode::NOT_FOUND.into_response()
}

async fn serve_image(path: PathBuf) -> Response {
    if let Ok(mut file) = File::open(&path).await {
        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).await.is_ok() {
            let mime = if path.extension().map_or(false, |e| e == "png") { "image/png" } else { "image/jpeg" };
            return (
                [(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "public, max-age=31536000, immutable")],
                buf
            ).into_response();
        }
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn play_album(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    let tracks = get_tracks_internal(&id, &state, None).await;
    state.mpd_engine.send(MpdCommand::Play { tracks, offset }).await;
    Json(json!({"status": "ok"})).into_response()
}

pub async fn play_disc(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let disc = params.get("disc").cloned();
    let tracks = get_tracks_internal(&id, &state, disc).await;
    state.mpd_engine.send(MpdCommand::Play { tracks, offset: 0 }).await;
    Json(json!({"status": "ok"})).into_response()
}

pub async fn queue_album(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let tracks = get_tracks_internal(&id, &state, None).await;
    state.mpd_engine.send(MpdCommand::Queue { tracks }).await;
    Json(json!({"status": "ok"})).into_response()
}

pub async fn next_track(State(state): State<Arc<AppState>>) -> Response {
    state.mpd_engine.send(MpdCommand::Next).await;
    Json(json!({"status": "ok"})).into_response()
}

pub async fn prev_track(State(state): State<Arc<AppState>>) -> Response {
    state.mpd_engine.send(MpdCommand::Prev).await;
    Json(json!({"status": "ok"})).into_response()
}

pub async fn stop_playback(State(state): State<Arc<AppState>>) -> Response {
    state.mpd_engine.send(MpdCommand::Stop).await;
    Json(json!({"status": "ok"})).into_response()
}

pub async fn clear_queue(State(state): State<Arc<AppState>>) -> Response {
    state.mpd_engine.send(MpdCommand::Clear).await;
    Json(json!({"status": "ok"})).into_response()
}

pub async fn toggle_pause(State(state): State<Arc<AppState>>) -> Response {
    state.mpd_engine.send(MpdCommand::TogglePause).await;
    Json(json!({"status": "ok"})).into_response()
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

async fn get_tracks_internal(id: &str, state: &Arc<AppState>, disc_filter: Option<String>) -> Vec<String> {
    let lib = state.library.read().await;
    let mut paths = Vec::new();
    if let Some(album) = lib.album_map.get(id) {
        for track in &album.tracks {
            if let Some(df) = &disc_filter {
                let d = track.other.get("DISCNUMBER").and_then(|v| v.as_str()).unwrap_or("1");
                if d != df { continue; }
            }
            if let Some(tp) = &track.track_library_path {
                if let Some(abs) = lib.track_map.get(tp) {
                    if let Ok(rel) = abs.strip_prefix(&state.config.library_root) {
                        if let Some(s) = rel.to_str() { paths.push(s.to_string()); }
                    }
                }
            }
        }
    }
    paths
}
