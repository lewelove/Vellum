use crate::server::AppState;
use mpd::{Client, Subsystem, Idle};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

pub fn start_monitor(state: Arc<AppState>) {
    let (tx_event, mut rx_event) = mpsc::channel::<()>(10);

    // Dedicated Idle Listener Thread
    // This thread owns its own connection and blocks on MPD events indefinitely.
    std::thread::spawn(move || {
        loop {
            match Client::connect("127.0.0.1:6600") {
                Ok(mut client) => {
                    log::info!("MPD Idle Listener connected.");
                    loop {
                        if client.wait(&[Subsystem::Player, Subsystem::Playlist, Subsystem::Options]).is_ok() {
                            let _ = tx_event.blocking_send(());
                        } else {
                            break;
                        }
                    }
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(2));
                }
            }
        }
    });

    // Main Async Monitor Task
    // This task manages status broadcasting and the 1Hz playback heartbeat.
    tokio::spawn(async move {
        log::info!("Starting Async Status Monitor...");
        let mut heartbeat = interval(Duration::from_secs(1));
        
        loop {
            let state_clone = state.clone();

            // Retrieve status and broadcast using a dedicated command connection
            let playback_active = tokio::task::spawn_blocking(move || {
                if let Ok(mut client) = Client::connect("127.0.0.1:6600") {
                    let status = client.status().ok();
                    let _ = broadcast_status(&mut client, &state_clone);
                    return status.map(|s| s.state == mpd::status::State::Play).unwrap_or(false);
                }
                false
            })
            .await
            .unwrap_or(false);

            if playback_active {
                // While playing, react to either the 1s pulse or an MPD event
                tokio::select! {
                    _ = heartbeat.tick() => {},
                    _ = rx_event.recv() => {
                        heartbeat = interval(Duration::from_secs(1));
                    }
                }
            } else {
                // While stopped/paused, only react to MPD events
                let _ = rx_event.recv().await;
                heartbeat = interval(Duration::from_secs(1));
            }
        }
    });
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
