use crate::server::logic::LogicEngine;
use anyhow::{Context, Result};
use mpd_client::Client;
use mpd_client::commands;
use mpd_client::responses::PlayState;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

pub async fn broadcast_status(
    client: &Client,
    tx: &broadcast::Sender<String>,
    logic: &Arc<RwLock<LogicEngine>>,
) -> Result<()> {
    let (status, current_song, queue) = client
        .command_list((commands::Status, commands::CurrentSong, commands::Queue))
        .await
        .context("Batched status update failed")?;

    let (file_path, title, artist) = current_song.as_ref().map_or_else(
        || (String::new(), None, None),
        |s| {
            (
                s.song.url.clone(),
                s.song.title().map(ToString::to_string),
                s.song.artists().first().map(ToString::to_string),
            )
        },
    );

    let (queue_json, album_id, track_index) = {
        let l = logic.read().await;
        let clean_file_path = file_path.trim_start_matches('/');
        let album_id = l
            .path_lookup
            .get(clean_file_path)
            .or_else(|| l.path_lookup.get(&file_path))
            .cloned();
        let track_index = l
            .track_lookup
            .get(clean_file_path)
            .or_else(|| l.track_lookup.get(&file_path))
            .and_then(|m| m.get("trackIndex"))
            .and_then(serde_json::Value::as_u64);
        let track_metas: Vec<Option<serde_json::Value>> = queue
            .iter()
            .map(|s| {
                let clean_url = s.song.url.trim_start_matches('/');
                l.track_lookup
                    .get(clean_url)
                    .or_else(|| l.track_lookup.get(&s.song.url))
                    .cloned()
            })
            .collect();
        drop(l);

        let q_json: serde_json::Value = queue
            .iter()
            .enumerate()
            .zip(track_metas)
            .map(|((idx, s), track_meta)| {
                track_meta.map_or_else(
                    || {
                        serde_json::json!({
                            "id": idx,
                            "file": s.song.url,
                            "title": s.song.title(),
                            "artist": s.song.artists().first(),
                            "album_id": serde_json::Value::Null,
                            "track_index": serde_json::Value::Null,
                            "track_no": serde_json::Value::Null,
                            "disc_no": 1,
                            "duration": "",
                            "duration_ms": 0,
                        })
                    },
                    |meta| {
                        serde_json::json!({
                            "id": idx,
                            "file": s.song.url,
                            "title": meta.get("title").cloned().or_else(|| s.song.title().map(|t| serde_json::json!(t))),
                            "artist": meta.get("artist").cloned().or_else(|| s.song.artists().first().map(|a| serde_json::json!(a))),
                            "album_id": meta.get("albumId"),
                            "track_index": meta.get("trackIndex"),
                            "track_no": meta.get("trackNo"),
                            "disc_no": meta.get("discNo"),
                            "duration": meta.get("duration"),
                            "duration_ms": meta.get("durationMs"),
                        })
                    },
                )
            })
            .collect();

        (q_json, album_id, track_index)
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
        "track_index": track_index,
        "elapsed": status.elapsed.map_or(0.0, |t| t.as_secs_f64()),
        "duration": status.duration.map_or(0.0, |t| t.as_secs_f64()),
        "title": title,
        "artist": artist,
        "queue": queue_json
    });

    let _ = tx.send(payload.to_string());
    Ok(())
}
