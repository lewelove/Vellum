File: rust/src/server/mpd/status.rs
Role: Playback Status Broadcaster

Description:
This file interrogates the music daemon for its current exact state (duration, current song, active playlist) and translates it into a universal JSON package sent to the web UI.

Imports:
`use crate::server::library::Library;`
- For cross-referencing tracks to albums.
`use mpd_client::responses::PlayState;`
- To determine if the engine is running or stopped.

Logic:
`pub async fn broadcast_status(client: &Client, tx: &broadcast::Sender<String>, library: &Arc<RwLock<Library>>) -> Result<()>`
- Collects and emits the live snapshot.
- It asks the daemon for three things at once: generic status, current playing song, and queue items. It extracts the raw file path of the currently playing song. It takes this path, searches the live memory library index, and finds the parent Album ID. Finally, it bundles the play state, times, titles, the queue array, and the Album ID into a single JSON packet and blasts it out across the WebSocket to all connected browser clients so their interfaces update flawlessly.
