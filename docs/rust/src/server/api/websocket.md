File: rust/src/server/api/websocket.rs
Role: Live Communication Hub

Description:
This file handles persistent, open connections between the server and the web browser, allowing the server to push real-time updates (like changing the play/pause state or updating a progress bar) instantly without the browser having to ask for it.

Imports:
`use crate::server::mpd::MpdCommand;`
- For refreshing player status upon connection.
`use crate::server::state::AppState;`
- For grabbing the library to send to the user.
`use ax_ws::WebSocket; use axum::extract::ws as ax_ws; use axum::extract::{State, WebSocketUpgrade};`
- For upgrading a normal HTTP connection into a continuous stream.
`use axum::response::Response;`
`use serde_json::json;`

Logic:
`pub async fn ws_handler(...) -> Response`
- Takes the incoming request and formally upgrades it into a continuous WebSocket stream.

`async fn handle_socket(...)`
- The infinite loop governing a single user's connection.
- The exact moment the user connects, it bundles up the entire library database and their personal UI preferences and blasts it to them so the page can render immediately. It asks MPD to provide an immediate status update. It then enters an infinite listening loop, waiting for any global broadcast events from the server (like track skips) and instantly feeding them down the pipe to the user until they close their browser.
