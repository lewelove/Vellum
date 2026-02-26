use ax_ws::WebSocket;
use axum::extract::ws as ax_ws;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use serde_json::json;
use std::sync::Arc;
use crate::server::state::AppState;
use crate::server::mpd::MpdCommand;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    log::info!("Client connected");

    let init_payload = {
        let lib_data = state.library.read().await.albums.clone();
        let ui_data = state.ui_state.read().await.clone();
        json!({
            "type": "INIT",
            "data": lib_data,
            "ui_state": ui_data
        }).to_string()
    };
    
    if socket.send(ax_ws::Message::Text(init_payload.into())).await.is_err() {
        return;
    }

    state.mpd_engine.send(MpdCommand::Refresh).await;

    let mut rx = state.tx.subscribe();
    loop {
        tokio::select! {
            Some(msg) = socket.recv() => {
                if let Ok(ax_ws::Message::Close(_)) | Err(_) = msg {
                    log::info!("Client disconnected");
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
