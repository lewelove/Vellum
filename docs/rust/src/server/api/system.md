File: rust/src/server/api/system.rs
Role: Administrative System Endpoints

Description:
This file manages high-level tasks like saving user interface preferences, forcing library resets, and opening native file explorers on the local machine.

Imports:
`use crate::server::state::AppState;`
- Access to the application database.
`use axum::Json; use axum::extract::{Path, Query, State}; use axum::http::StatusCode; use axum::response::{IntoResponse, Response};`
- Web handling elements.
`use serde_json::json;`
- Used to format responses easily.

Logic:
`pub async fn update_state(...) -> Response`
- Receives UI preferences (like sort order and active filters) from the browser, updates the live memory, and instantly writes them to `state.json` on the disk so they persist across reboots.

`pub async fn trigger_full_reset(...) -> Response`
- The nuclear option for library issues. It clears the entire live database in RAM, runs a brand new scan of the hard drive, and violently pushes the completely refreshed library to all connected clients over the WebSocket.

`pub async fn trigger_reload(...) -> Response`
- A surgical update tool. If an album is recompiled in the background, this receives the folder path, opens the specific file, overrides the old entry in RAM, and pushes just that single updated album to the frontend to seamlessly refresh the view.

`pub async fn open_album_folder(...) / open_lock_file(...) / open_manifest_file(...) -> Response`
- Spawns a background command asking the operating system's native file explorer to open the requested directory or text file directly. Excellent for immediate desktop management.

`pub async fn force_update_album(...) -> Response`
- Spawns a background terminal command to manually force the compiler engine to rebuild a specific album completely from scratch.
