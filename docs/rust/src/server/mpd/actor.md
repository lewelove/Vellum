File: rust/src/server/mpd/actor.rs
Role: Audio Player Actor

Description:
This file governs the connection between the Vellum application and the underlying Music Player Daemon (MPD). It runs endlessly in the background, receiving commands from the interface and listening to player events.

Imports:
`use crate::server::library::Library; use crate::server::mpd::commands::{MpdCommand, handle_command}; use crate::server::mpd::status::broadcast_status;`
- Connects to necessary internal modules.
`use mpd_client::Client; use mpd_client::client::{ConnectionEvent, Subsystem};`
- The external framework for interacting with MPD over a network.
`use tokio::net::TcpStream; use tokio::sync::{RwLock, broadcast, mpsc};`
- Asynchronous networking and thread messaging tools.

Logic:
`pub struct MpdEngine`
- A small messenger object handed to the rest of the application so they can send tasks to this background actor.

`pub async fn send(&self, command: MpdCommand)`
- Submits a command into the actor's asynchronous queue.

`pub fn start_actor(...) -> MpdEngine`
- Bootstraps and sustains the daemon connection.
- It spawns an infinite, isolated background thread. It establishes a TCP connection to the MPD host port. Upon successful connection, it expands the daemon's internal data limits (to handle heavy queues) and enters an event loop. The loop uses `tokio::select!` to listen for two things simultaneously: incoming commands from the web UI (like skipping tracks) and natural events emitted by the player (like a song ending or pausing). Whenever an action occurs, it broadcasts the brand-new playback status globally. If the connection drops, it catches the error and automatically loops back to reconnect.
