use crate::server::library::Library;
use anyhow::{Context, Result};
use mpd::Client;
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

pub fn broadcast_status(
    client: &mut Client<TcpStream>,
    tx: &broadcast::Sender<String>,
    library: &Arc<RwLock<Library>>,
) -> Result<()> {
    let status = client.status().context("Status failed")?;
    let current_song = client.currentsong().context("Current song failed")?;
    let queue = client.queue().context("Queue failed")?;

    let (file_path, title, artist) = if let Some(s) = current_song {
        let path = s.file.trim_start_matches('/').to_string();
        let t = s.title.clone().or_else(|| get_tag(&s, "Title"));
        let a = get_tag(&s, "Artist");
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
                "file": s.file.trim_start_matches('/').to_string(),
                "title": s.title.clone().or_else(|| get_tag(s, "Title")),
                "artist": get_tag(s, "Artist"),
            })
        })
        .collect();

    let album_id = {
        let lib = library.blocking_read();
        lib.path_lookup.get(&file_path).cloned()
    };

    let payload = serde_json::json!({
        "type": "MPD_STATUS",
        "state": format!("{:?}", status.state).to_lowercase(),
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

fn get_tag(song: &mpd::song::Song, key: &str) -> Option<String> {
    song.tags
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v.clone())
}
