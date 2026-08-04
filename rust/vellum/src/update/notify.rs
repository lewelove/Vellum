use crate::compile;
use crate::update::cache::FileStat;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

pub struct NotificationTaskArgs {
    pub notify_rx: mpsc::Receiver<compile::stream::AlbumUpdateSignal>,
    pub cache_for_task: Arc<Mutex<HashMap<String, FileStat>>>,
    pub lib_root_for_task: Arc<PathBuf>,
    pub missing_paths: Vec<PathBuf>,
    pub start_time: std::time::Instant,
    pub silent: bool,
}

pub fn start_notification_task(args: NotificationTaskArgs) -> tokio::task::JoinHandle<()> {
    let mut rx = args.notify_rx;
    let cache_for_task = args.cache_for_task;
    let lib_root_for_task = args.lib_root_for_task;
    let missing_paths = args.missing_paths;
    let start_time = args.start_time;
    let silent = args.silent;

    tokio::spawn(async move {
        let mut updated_paths = Vec::new();
        while let Some(signal) = rx.recv().await {
            if !silent {
                log::info!("Updated: {} - {}", signal.artist, signal.album);
            }
            updated_paths.push(signal.path);
        }

        let mut paths_for_server = Vec::new();

        {
            let mut c = cache_for_task.lock().await;
            for album_root in &updated_paths {
                let album_id = libvellum::resolvers::rel_path(album_root, &lib_root_for_task);

                for entry in walkdir::WalkDir::new(album_root)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(Result::ok)
                {
                    let p = entry.path();
                    if p.is_file() {
                        let rel = libvellum::resolvers::rel_path(p, &lib_root_for_task);
                        if let Ok(m) = entry.metadata() {
                            let mtime = m
                                .modified()
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let size = m.len();
                            c.insert(rel, FileStat { mtime, size });
                        }
                    }
                }
                paths_for_server.push(album_id);
            }

            for missing in missing_paths {
                let album_id = libvellum::resolvers::rel_path(&missing, &lib_root_for_task);
                let prefix = format!("{album_id}/");

                if !silent {
                    let display_path = missing.strip_prefix(&*lib_root_for_task).unwrap_or(&missing);
                    log::info!("Removed: {}", display_path.display());
                }

                c.retain(|k, _| !k.starts_with(&prefix));
                paths_for_server.push(album_id);
            }
            drop(c);
        }

        if paths_for_server.is_empty() {
            return;
        }

        let elapsed = start_time.elapsed().as_millis();
        let client = reqwest::Client::new();
        let _ = client
            .post("http://127.0.0.1:8000/api/internal/batch_reload")
            .query(&[("time", elapsed.to_string())])
            .json(&paths_for_server)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;
    })
}
