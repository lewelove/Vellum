use crate::server::AppState;
use mpd::{Client, Subsystem, Idle};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

pub fn start_monitor(state: Arc<AppState>) {
    tokio::spawn(async move {
        log::info!("Starting MPD Monitor...");

        let mut heartbeat = interval(Duration::from_secs(1));

        loop {
            let state_clone = state.clone();

            let playback_active = tokio::task::spawn_blocking(move || {
                match Client::connect("127.0.0.1:6600") {
                    Ok(mut client) => {
                        let status = client.status().ok();
                        let _ = broadcast_status(&mut client, &state_clone);
                        status.map(|s| s.state == mpd::status::State::Play).unwrap_or(false)
                    }
                    Err(_) => false,
                }
            })
            .await
            .unwrap_or(false);

            if playback_active {
                tokio::select! {
                    _ = heartbeat.tick() => {},
                    _ = wait_for_event() => {
                        heartbeat = interval(Duration::from_secs(1));
                    }
                }
            } else {
                let _ = wait_for_event().await;
                heartbeat = interval(Duration::from_secs(1));
            }
        }
    });
}

async fn wait_for_event() -> Result<(), ()> {
    tokio::task::spawn_blocking(move || {
        if let Ok(mut client) = Client::connect("127.0.0.1:6600") {
            let _ = client.wait(&[Subsystem::Player, Subsystem::Playlist, Subsystem::Options]);
            Ok(())
        } else {
            std::thread::sleep(Duration::from_secs(2));
            Err(())
        }
    })
    .await
    .unwrap_or(Err(()))
}

fn broadcast_status(client: &mut Client, state: &Arc<AppState>) -> Result<(), mpd::error::Error> {
    let status = client.status()?;
    let current_song = client.currentsong()?;
    let queue = client.queue()?;

    let file_path = current_song.as_ref().map(|s| s.file.clone()).unwrap_or_default();
    
    let album_id = {
        let lib = state.library.blocking_read();
        let norm_path = file_path.trim_start_matches('/');
        lib.path_lookup.get(norm_path).cloned()
    };

    let payload = json!({
        "type": "MPD_STATUS",
        "state": format!("{:?}", status.state).to_lowercase(),
        "file": file_path,
        "album_id": album_id,
        "elapsed": status.elapsed.map(|t| t.as_secs_f64()).unwrap_or(0.0),
        "duration": status.duration.map(|t| t.as_secs_f64()).unwrap_or(0.0),
        "title": current_song.as_ref().and_then(|s| s.title.as_ref()).map(|s| s.as_str()).unwrap_or(""),
        "artist": current_song.as_ref().and_then(|s| s.artist.as_ref()).map(|s| s.as_str()).unwrap_or(""),
        "queue": queue.iter().enumerate().map(|(idx, s)| json!({
            "id": idx, 
            "file": s.file,
            "title": s.title,
            "artist": s.artist,
            "albumartist": s.tags.iter().find(|(k, _)| k == "AlbumArtist").map(|(_, v)| v),
        })).collect::<Vec<_>>()
    });

    let _ = state.tx.send(payload.to_string());
    Ok(())
}

fn get_mpd_conn() -> Result<Client, mpd::error::Error> {
    Client::connect("127.0.0.1:6600")
}

pub fn play_paths(paths: Vec<String>, offset: usize) -> bool {
    if paths.is_empty() { return false; }
    
    if let Ok(mut client) = get_mpd_conn() {
        let _ = client.clear();
        for path in paths {
            let s = mpd::song::Song {
                file: path,
                last_mod: None,
                name: None,
                title: None,
                artist: None,
                duration: None,
                place: None,
                range: None,
                tags: Vec::new(),
            };
            let _ = client.push(s);
        }
        let _ = client.switch(offset as u32);
        return true;
    }
    false
}

pub fn enqueue_paths(paths: Vec<String>) -> bool {
    if paths.is_empty() { return false; }
    if let Ok(mut client) = get_mpd_conn() {
        for path in paths {
            let s = mpd::song::Song {
                file: path,
                last_mod: None,
                name: None,
                title: None,
                artist: None,
                duration: None,
                place: None,
                range: None,
                tags: Vec::new(),
            };
            let _ = client.push(s);
        }
        return true;
    }
    false
}
