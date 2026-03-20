File: rust/src/server/api/playback.rs
Role: Playback Remote Controller

Description:
This file intercepts requests from the UI (like hitting the 'Play' button) and instructs the background audio player (MPD) on what to do.

Imports:
`use crate::server::mpd::MpdCommand;`
- Brings in the vocabulary of actions the player understands.
`use crate::server::state::AppState;`
- Gives access to the live library and player connection.
`use axum::Json; use axum::extract::{Path, Query, State};`
- For parsing web requests and URL query strings.
`use axum::response::{IntoResponse, Response};`
- For replying to the interface.

Logic:
`pub async fn play_album(...) -> Response`
- Initiates immediate playback of a specific album.
- Looks up the tracks belonging to an album, sends a command to MPD to clear everything else, inserts the tracks, and starts playing them immediately (optionally starting at a specific track offset).

`pub async fn play_disc(...) -> Response`
- Plays only a specific disc of a larger album.
- Similar to `play_album`, but uses a filter to only queue up tracks that match the requested disc number.

`pub async fn queue_album(...) -> Response`
- Appends an album to the end of the current playlist silently without stopping whatever is currently playing.

`pub async fn next_track(...) / prev_track(...) / stop_playback(...) / clear_queue(...) / toggle_pause(...) -> Response`
- Simple straightforward functions that act like buttons on a remote control, sending identical commands straight to the player engine.

`async fn get_tracks_internal(...) -> Vec<String>`
- Translates a generic Album ID into raw file paths.
- Because MPD only understands physical file paths, this helper dives into the RAM database, retrieves the album, extracts the specific paths for every audio file inside it, and hands them back as a simple list.
