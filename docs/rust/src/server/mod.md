File: rust/src/server/mod.rs
Role: Server Bootstrapper

Description:
This is the heart of the backend application. It wires together the configuration, the database scanner, the MPD background actor, and the web HTTP router into a unified process.

Imports:
`use anyhow::{Context, Result};`
`use std::net::SocketAddr; use std::sync::Arc; use tokio::sync::{RwLock, broadcast}; use tower_http::cors::{Any, CorsLayer};`
- Framework dependencies for networking, threading, and web permissions.

Logic:
`pub async fn run(port: u16) -> Result<()>`
- The ignition sequence for the server.
- It loads the user's configuration and establishes fundamental path rules. It reads the local `.vellum/state.json` file to restore the UI's last known filter preferences. It initializes the `Library` object and triggers a total scan of the hard drive into RAM. It spawns the MPD connection actor. It bundles the library, the settings, and the actors into an `AppState` container, wraps it in a thread-safe `Arc`, and attaches it to the HTTP router. It configures lenient CORS rules (allowing the separate dev UI to talk to it). Lastly, it binds a TCP socket to the specified port and launches the infinite web-server loop to handle traffic.
