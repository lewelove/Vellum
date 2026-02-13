use crate::server::library::Library;
use crate::server::AppConfig;
use anyhow::{Context, Result};
use mpd::{Client, Idle, Subsystem};
use std::io::Write;
use std::net::TcpStream;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio::sync::{broadcast, oneshot, RwLock};

pub enum MpdRequest {
    Execute(Box<dyn FnOnce(&mut Client<TcpStream>) -> Result<()> + Send>),
    #[allow(dead_code)]
    Query(
        Box<dyn FnOnce(&mut Client<TcpStream>) -> Result<serde_json::Value> + Send>,
        oneshot::Sender<Result<serde_json::Value>>,
    ),
}

impl std::fmt::Debug for MpdRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Execute(_) => write!(f, "MpdRequest::Execute"),
            Self::Query(_, _) => write!(f, "MpdRequest::Query"),
        }
    }
}

#[derive(Clone)]
pub struct MpdEngine {
    tx: mpsc::Sender<MpdRequest>,
    stream_interrupt: Arc<parking_lot::Mutex<Option<TcpStream>>>,
}

impl MpdEngine {
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&mut Client<TcpStream>) -> Result<()> + Send + 'static,
    {
        if let Some(stream) = self.stream_interrupt.lock().as_mut() {
            let _ = stream.write_all(b"noidle\n");
            let _ = stream.flush();
        }
        let _ = self.tx.send(MpdRequest::Execute(Box::new(f)));
    }

    #[allow(dead_code)]
    pub async fn query<F>(&self, f: F) -> Result<serde_json::Value>
    where
        F: FnOnce(&mut Client<TcpStream>) -> Result<serde_json::Value> + Send + 'static,
    {
        let (res_tx, res_rx) = oneshot::channel();
        if let Some(stream) = self.stream_interrupt.lock().as_mut() {
            let _ = stream.write_all(b"noidle\n");
            let _ = stream.flush();
        }
        self.tx
            .send(MpdRequest::Query(Box::new(f), res_tx))
            .map_err(|_| anyhow::anyhow!("Failed to send query request to MPD worker"))?;
        res_rx
            .await
            .map_err(|_| anyhow::anyhow!("MPD worker dropped the query response channel"))?
    }
}

pub fn start_monitor(
    broadcast_tx: broadcast::Sender<String>,
    library: Arc<RwLock<Library>>,
    config: Arc<AppConfig>,
) -> MpdEngine {
    let (tx, rx) = mpsc::channel::<MpdRequest>();
    let stream_interrupt = Arc::new(parking_lot::Mutex::new(None));
    let engine_handle = MpdEngine {
        tx,
        stream_interrupt: Arc::clone(&stream_interrupt),
    };

    let mpd_host = std::env::var("MPD_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let mpd_port = std::env::var("MPD_PORT").unwrap_or_else(|_| "6600".to_string());
    let connection_str = format!("{}:{}", mpd_host, mpd_port);

    std::thread::spawn(move || {
        loop {
            match TcpStream::connect(&connection_str) {
                Ok(stream) => {
                    if let Ok(stream_clone) = stream.try_clone() {
                        *stream_interrupt.lock() = Some(stream_clone);
                    }

                    match Client::new(stream) {
                        Ok(mut client) => {
                            log::info!("MPD connected via single persistent channel.");

                            loop {
                                if let Err(e) =
                                    broadcast_status(&mut client, &broadcast_tx, &library, &config)
                                {
                                    log::error!("Status broadcast failed: {}", e);
                                    break;
                                }

                                match rx.recv_timeout(Duration::from_secs(1)) {
                                    Ok(request) => handle_request(request, &mut client),
                                    Err(mpsc::RecvTimeoutError::Timeout) => {
                                        let _ = client.wait(&[
                                            Subsystem::Player,
                                            Subsystem::Playlist,
                                            Subsystem::Options,
                                            Subsystem::Mixer,
                                        ]);
                                    }
                                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("MPD client initialization failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("TCP Connection to MPD failed: {}. Retrying...", e);
                }
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });

    engine_handle
}

fn handle_request(request: MpdRequest, client: &mut Client<TcpStream>) {
    match request {
        MpdRequest::Execute(f) => {
            if let Err(e) = f(client) {
                log::error!("MPD Command failed: {}", e);
            }
        }
        MpdRequest::Query(f, tx) => {
            let _ = tx.send(f(client));
        }
    }
}

fn broadcast_status(
    client: &mut Client<TcpStream>,
    tx: &broadcast::Sender<String>,
    library: &Arc<RwLock<Library>>,
    _config: &Arc<AppConfig>,
) -> Result<()> {
    let status = client.status().context("Failed to get status")?;
    let current_song = client.currentsong().context("Failed to get current song")?;
    let queue = client.queue().context("Failed to get queue")?;

    let file_path = current_song
        .as_ref()
        .map(|s| s.file.trim_start_matches('/').to_string())
        .unwrap_or_default();

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
        "queue": queue.iter().enumerate().map(|(idx, s)| {
            let normalized_file = s.file.trim_start_matches('/').to_string();
            serde_json::json!({
                "id": idx,
                "file": normalized_file,
            })
        }).collect::<Vec<_>>()
    });

    let _ = tx.send(payload.to_string());
    Ok(())
}
