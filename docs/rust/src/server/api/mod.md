File: rust/src/server/api/mod.rs
Role: Web API Router

Description:
This file is the switchboard operator for the backend. It maps specific web addresses (like `/api/play/...`) to the underlying Rust functions that actually perform the logic.

Imports:
`pub mod assets; pub mod playback; pub mod system; pub mod websocket;`
- Connects the individual endpoint modules.
`use crate::server::state::AppState;`
- Brings in the master server state.
`use axum::{Router, routing::{get, post}};`
- The core framework elements for mapping web paths.
`use std::sync::Arc;`
- For safely sharing the state reference.

Logic:
`pub fn router(state: Arc<AppState>) -> Router`
- Wires up the web endpoints.
- It creates a new Axum router, defining whether a path expects a GET or POST request, links it to its respective handler function, injects the shared state, and establishes basic cross-origin resource sharing (CORS) rules so the separate web frontend can communicate with it safely.
