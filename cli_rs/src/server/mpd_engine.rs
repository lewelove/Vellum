use crate::server::library::Library;
use crate::server::AppConfig;
use anyhow::{Context, Result};
use mpd::{Client, Idle, Subsystem};
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

pub enum MpdCommand {
    Play { tracks: Vec<String>, offset: usize },
    Queue { tracks: Vec<String> },
    Clear,
    Stop,
    Next,
    Prev,
    TogglePause,
    Refresh,
}

#[derive(Clone)]
pub struct MpdEngine {
    tx: mpsc::Sender<MpdCommand>,
    interrupt_stream: Arc<parking_lot::Mutex<Option<TcpStream>>>,
}

impl MpdEngine {
    pub async fn send(&self, command: MpdCommand) {
        if let Some(stream) = self.interrupt_stream.lock().as_mut() {
            let _ = stream.write_all(b"noidle\n");
            let _ = stream.flush();
        }
        let _ = self.tx.send(command).await;
    }
}

pub fn start_actor(
    broadcast_tx: broadcast::Sender<String>,
    library: Arc<RwLock<Library>>,
    _app_config: Arc<AppConfig>,
) -> MpdEngine {
    let (tx, mut rx) = mpsc::channel::<MpdCommand>(32);
    let interrupt_stream = Arc::new(parking_lot::Mutex::new(None));
    let engine_handle = MpdEngine {
        tx,
        interrupt_stream: Arc::clone(&interrupt_stream),
    };

    let mpd_host = std::env::var("MPD_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let mpd_port = std::env::var("MPD_PORT").unwrap_or_else(|_| "6600".to_string());
    let addr = format!("{}:{}", mpd_host, mpd_port);

    std::thread::spawn(move || {
        loop {
            match TcpStream::connect(&addr) {
                Ok(stream) => {
                    if let Ok(clone) = stream.try_clone() {
                        *interrupt_stream.lock() = Some(clone);
                    }

                    if let Ok(mut client) = Client::new(stream) {
                        log::info!("MPD Actor connected and ready.");
                        
                        loop {
                            if let Err(e) = broadcast_status(&mut client, &broadcast_tx, &library) {
                                log::error!("MPD broadcast error: {}", e);
                                break;
                            }

                            // Block on Idle until a subsystem changes or noidle is received
                            let _ = client.wait(&[
                                Subsystem::Player,
                                Subsystem::Playlist,
                                Subsystem::Options,
                                Subsystem::Mixer,
                            ]);

                            // Drain all commands received while idling or interrupted
                            while let Ok(cmd) = rx.try_recv() {
                                if let Err(e) = handle_command(&mut client, cmd) {
                                    log::error!("MPD Command execution error: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("MPD Connection failed: {}. Retrying in 2s...", e);
                }
            }
            *interrupt_stream.lock() = None;
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    engine_handle
}

fn handle_command(client: &mut Client<TcpStream>, cmd: MpdCommand) -> Result<()> {
    match cmd {
        MpdCommand::Play { tracks, offset } => {
            client.clear()?;
            for track in tracks {
                client.push(mpd::song::Song {
                    file: track,
                    ..Default::default()
                })?;
            }
            client.switch(offset as u32)?;
        }
        MpdCommand::Queue { tracks } => {
            for track in tracks {
                client.push(mpd::song::Song {
                    file: track,
                    ..Default::default()
                })?;
            }
        }
        MpdCommand::Clear => client.clear()?,
        MpdCommand::Stop => client.stop()?,
        MpdCommand::Next => client.next()?,
        MpdCommand::Prev => client.prev()?,
        MpdCommand::TogglePause => {
            let status = client.status()?;
            match status.state {
                mpd::status::State::Play => client.pause(true)?,
                mpd::status::State::Pause => client.pause(false)?,
                mpd::status::State::Stop => client.play()?,
            }
        }
        MpdCommand::Refresh => {} 
    }
    Ok(())
}

fn broadcast_status(
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
        "elapsed": status.elapsed.map(|t| t.as_secs_f64()).unwrap_or(0.0),
        "duration": status.duration.map(|t| t.as_secs_f64()).unwrap_or(0.0),
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
