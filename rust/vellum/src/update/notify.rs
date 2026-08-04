use crate::compile;
use crate::update::cache::{AlbumCacheEntry, get_mtime_sum};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

pub struct NotificationTaskArgs {
    pub notify_rx: mpsc::Receiver<compile::stream::AlbumUpdateSignal>,
    pub cache_for_task: Arc<Mutex<HashMap<String, AlbumCacheEntry>>>,
    pub exts_for_task: Vec<String>,
    pub manifests_for_task: Option<Vec<String>>,
    pub lib_root_for_task: Arc<PathBuf>,
    pub missing_paths: Vec<PathBuf>,
    pub start_time: std::time::Instant,
    pub silent: bool,
}

pub fn start_notification_task(args: NotificationTaskArgs) -> tokio::task::JoinHandle<()> {
    let mut rx = args.notify_rx;
    let cache_for_task = args.cache_for_task;
    let exts_for_task = args.exts_for_task;
    let manifests_for_task = args.manifests_for_task;
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
                let album_path_str = album_root.to_string_lossy().to_string();
                let metadata_path = album_root.join("metadata.toml");
                let mtime_sum = get_mtime_sum(album_root, &metadata_path, &exts_for_task, manifests_for_task.as_ref());
                c.insert(album_path_str.clone(), AlbumCacheEntry { mtime_sum });
                paths_for_server.push(album_path_str);
            }

            for missing in missing_paths {
                let p_str = missing.to_string_lossy().to_string();

                if !silent {
                    let display_path = missing.strip_prefix(&*lib_root_for_task).unwrap_or(&missing);
                    log::info!("Removed: {}", display_path.display());
                }

                c.remove(&p_str);
                paths_for_server.push(p_str);
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
