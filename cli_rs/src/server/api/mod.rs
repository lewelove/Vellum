pub mod websocket;
pub mod playback;
pub mod assets;
pub mod system;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::server::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(websocket::ws_handler))
        .route("/api/state", post(system::update_state))
        .route("/api/internal/reset", post(system::trigger_full_reset))
        .route("/api/internal/reload", post(system::trigger_reload))
        .route("/api/covers/{hash}", get(assets::get_cover_thumbnail))
        .route("/api/assets/cover/{*id}", get(assets::get_album_cover))
        .route("/api/play/{*id}", post(playback::play_album))
        .route("/api/play-disc/{*id}", post(playback::play_disc))
        .route("/api/queue/{*id}", post(playback::queue_album))
        .route("/api/next", post(playback::next_track))
        .route("/api/prev", post(playback::prev_track))
        .route("/api/stop", post(playback::stop_playback))
        .route("/api/clear", post(playback::clear_queue))
        .route("/api/toggle-pause", post(playback::toggle_pause))
        .route("/api/open/{*id}", post(system::open_album_folder))
        .with_state(state)
}
