use crate::server::library::Library;
use anyhow::{Context, Result};
use mpd_client::Client;
use mpd_client::commands;
use mpd_client::responses::PlayState;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

pub async fn broadcast_status(
    client: &Client,
    tx: &broadcast::Sender<String>,
    library: &Arc<RwLock<Library>>,
) -> Result<()> {
    let status = client.command(commands::Status).await.context("Status failed")?;
    let current_song = client.command(commands::CurrentSong).await.context("Current song failed")?;
    let queue = client.command(commands::Queue).await.context("Queue failed")?;

    let (file_path, title, artist) = if let Some(s) = current_song {
        let path = s.song.url.clone();
        let t = s.song.title().map(ToString::to_string);
        let a = s.song.artists().first().map(ToString::to_string);
        (path, t, a)
    } else {
        (String::new(), None, None)
    };

    let queue_json: serde_json::Value = queue
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            serde_json::json!({
                "id": idx,
                "file": s.song.url,
                "title": s.song.title(),
                "artist": s.song.artists().first(),
            })
        })
        .collect();

    let album_id = {
        let lib = library.read().await;
        lib.path_lookup.get(&file_path).cloned()
    };

    let state_str = match status.state {
        PlayState::Playing => "play",
        PlayState::Paused => "pause",
        PlayState::Stopped => "stop",
    };

    let payload = serde_json::json!({
        "type": "MPD_STATUS",
        "state": state_str,
        "file": file_path,
        "album_id": album_id,
        "elapsed": status.elapsed.map_or(0.0, |t| t.as_secs_f64()),
        "duration": status.duration.map_or(0.0, |t| t.as_secs_f64()),
        "title": title,
        "artist": artist,
        "queue": queue_json
    });

    let _ = tx.send(payload.to_string());
    Ok(())
}
