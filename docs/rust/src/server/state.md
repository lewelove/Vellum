File: rust/src/server/state.rs
Role: Application State Definition

Description:
This small file defines the global objects that are shared simultaneously across hundreds of asynchronous web requests, serving as the connective tissue of the server.

Imports:
`use crate::server::library::Library; use crate::server::mpd::MpdEngine;`
`use std::path::PathBuf; use std::sync::Arc; use tokio::sync::{RwLock, broadcast};`

Logic:
`pub struct AppState`
- Holds the global resources. It contains the `Library` (wrapped in a read/write lock so multiple threads can read it while the updater safely writes to it), the raw JSON of the UI preferences, the `broadcast::Sender` used to throw events to WebSockets, the static configuration rules, and the handle to the background MPD actor.

`pub struct AppConfig`
- A miniaturized chunk of the global configuration strictly necessary for the server's operation, preventing excessive memory bloat.
