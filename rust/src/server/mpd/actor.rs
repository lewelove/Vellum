use crate::server::library::Library;
use crate::server::mpd::commands::{MpdCommand, handle_command};
use crate::server::mpd::status::broadcast_status;
use crate::server::state::AppConfig;
use mpd::{Client, Idle, Subsystem};
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};

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
    let addr = format!("{mpd_host}:{mpd_port}");

    std::thread::spawn(move || {
        loop {
            match TcpStream::connect(&addr) {
                Ok(stream) => {
                    if let Ok(clone) = stream.try_clone() {
                        *interrupt_stream.lock() = Some(clone);
                    }

                    if let Ok(mut client) = Client::new(stream) {
                        log::info!("MPD Connected: {addr}");
                        loop {
                            if broadcast_status(&mut client, &broadcast_tx, &library).is_err() {
                                break;
                            }
                            let _ = client.wait(&[
                                Subsystem::Player,
                                Subsystem::Playlist,
                                Subsystem::Options,
                            ]);
                            while let Ok(cmd) = rx.try_recv() {
                                match &cmd {
                                    MpdCommand::Play { tracks, .. } => {
                                        log::info!("Playing album ({} tracks)", tracks.len());
                                    }
                                    MpdCommand::Queue { tracks } => {
                                        log::info!("Enqueuing {} tracks", tracks.len());
                                    }
                                    MpdCommand::Next => log::info!("Skip next"),
                                    MpdCommand::Prev => log::info!("Skip previous"),
                                    MpdCommand::TogglePause => log::info!("Toggle pause"),
                                    MpdCommand::Clear => log::info!("Clear queue"),
                                    MpdCommand::Stop => log::info!("Stop playback"),
                                    MpdCommand::Refresh => {}
                                }

                                if let Err(e) = handle_command(&mut client, cmd) {
                                    log::error!("MPD Execution Error: {e}");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("MPD Connection Failed: {e}");
                }
            }
            *interrupt_stream.lock() = None;
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    engine_handle
}
