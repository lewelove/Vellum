use axum::extract::{Path, State, Query};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use crate::server::state::AppState;
use crate::server::mpd::MpdCommand;

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
