use axum::extract::ws as ax_ws;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use axum::http::{StatusCode, header};
use tokio::fs::File;

use crate::server::AppState;

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

    let state_dir = crate::expand_path("~/.vellum");
    if let Ok(content) = serde_json::to_string_pretty(&*state.ui_state.read().await) {
        let state_file = state_dir.join("state.json");
        let _ = std::fs::write(state_file, content);
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
    Json(json!({"status": "reset_complete"})).into_response()
}

#[derive(Deserialize)]
pub struct PathQuery {
    path: String,
}

pub async fn trigger_reload(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<PathQuery>,
) -> Response {
    let mut lib = state.library.write().await;
    if let Some(updated) = lib.update_album(&params.path) {
        let payload = json!({
            "type": "UPDATE",
            "id": updated.id,
            "payload": updated
        }).to_string();
        let _ = state.tx.send(payload);
        return Json(json!({"status": "reloaded"})).into_response();
    }
    (StatusCode::NOT_FOUND, Json(json!({"status": "not_found"}))).into_response()
}

async fn serve_image(path: PathBuf) -> Response {
    if !path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    
    match File::open(&path).await {
        Ok(mut file) => {
            let mut contents = Vec::new();
            if file.read_to_end(&mut contents).await.is_ok() {
                let mime = if let Some(ext) = path.extension() {
                    let e = ext.to_str().unwrap_or("").to_lowercase();
                    if e == "png" { "image/png" } else { "image/jpeg" }
                } else {
                    "application/octet-stream"
                };

                return (
                    [
                        (header::CONTENT_TYPE, mime),
                        (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                    ],
                    contents
                ).into_response();
            }
        }
        Err(_) => {}
    }
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
        if let Some(album) = lib.album_map.get(&id) {
            album.album_data.cover_path.as_ref().and_then(|cp| {
                 if cp != "default_cover.png" {
                     Some(state.config.library_root.join(&id).join(cp))
                 } else { None }
            })
        } else { None }
    };

    if let Some(path) = path_opt {
        return serve_image(path).await;
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn play_album(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let offset: usize = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    let paths = get_album_tracks_internal(&id, &state, None).await;
    
    state.mpd_manager.execute(move |client| {
        client.clear()?;
        for path in paths {
            client.push(mpd::song::Song {
                file: path,
                ..Default::default()
            })?;
        }
        client.switch(offset as u32)?;
        Ok(())
    });

    Json(json!({"status": "ok"})).into_response()
}

pub async fn play_disc(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let disc_opt = params.get("disc").cloned();
    let paths = get_album_tracks_internal(&id, &state, disc_opt).await;

    state.mpd_manager.execute(move |client| {
        client.clear()?;
        for path in paths {
            client.push(mpd::song::Song {
                file: path,
                ..Default::default()
            })?;
        }
        client.play()?;
        Ok(())
    });

    Json(json!({"status": "ok"})).into_response()
}

pub async fn queue_album(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let paths = get_album_tracks_internal(&id, &state, None).await;

    state.mpd_manager.execute(move |client| {
        for path in paths {
            client.push(mpd::song::Song {
                file: path,
                ..Default::default()
            })?;
        }
        Ok(())
    });

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
    (StatusCode::NOT_FOUND, Json(json!({"error": "path not found"}))).into_response()
}

async fn get_album_tracks_internal(id: &str, state: &Arc<AppState>, disc_filter: Option<String>) -> Vec<String> {
    let lib = state.library.read().await;
    let mut result = Vec::new();
    
    if let Some(album) = lib.album_map.get(id) {
        for track in &album.tracks {
            if let Some(d) = &disc_filter {
                let track_disc = track.other.get("DISCNUMBER").and_then(|v| v.as_str()).unwrap_or("1");
                if track_disc != d { continue; }
            }

            if let Some(t_lib_path) = &track.track_library_path {
                if let Some(abs_path) = lib.track_map.get(t_lib_path) {
                    if let Ok(rel) = abs_path.strip_prefix(&state.config.library_root) {
                         if let Some(s) = rel.to_str() {
                             result.push(s.to_string());
                         }
                    }
                }
            }
        }
    }
    result
}
