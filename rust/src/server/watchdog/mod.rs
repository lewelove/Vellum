use crate::config::AppConfig;
use crate::server::state::AppState;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub fn start(config_path: PathBuf, state: Arc<AppState>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let config_dir = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();

    tokio::spawn(async move {
        let tx_clone = tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() {
                        let _ = tx_clone.blocking_send(());
                    }
                }
            },
            notify::Config::default(),
        )
        .expect("Failed to create config watcher");

        watcher
            .watch(&config_path, RecursiveMode::NonRecursive)
            .expect("Failed to watch config file");

        let mut current_watched_shader: Option<PathBuf> = None;

        let initial_shader_path = {
            let guard = state.config.resolved_shader_path.read().await;
            guard.clone()
        };

        if let Some(ref p) = initial_shader_path {
            if p.exists() {
                let _ = watcher.watch(p, RecursiveMode::NonRecursive);
                current_watched_shader = Some(p.clone());
            }
        }

        while rx.recv().await.is_some() {
            tokio::time::sleep(Duration::from_millis(100)).await;
            while rx.try_recv().is_ok() {}

            log::info!("Filesystem change: reloading config...");

            match AppConfig::load() {
                Ok((new_config, _, _)) => {
                    let thumb_size = new_config.theme.as_ref().map_or(200, |t| t.thumbnail_size);
                    let shader_cfg = new_config.theme.as_ref().and_then(|t| t.shader.clone());

                    let mut resolved_path = None;
                    if let Some(ref s) = shader_cfg 
                        && let Some(ref p) = s.path 
                    {
                        let p_buf = PathBuf::from(p);
                        resolved_path = if p_buf.is_absolute() {
                            Some(p_buf)
                        } else {
                            Some(config_dir.join(p_buf))
                        };
                    }

                    if resolved_path != current_watched_shader {
                        if let Some(ref old_p) = current_watched_shader {
                            let _ = watcher.unwatch(old_p);
                        }
                        if let Some(ref new_p) = resolved_path {
                            if new_p.exists() {
                                let _ = watcher.watch(new_p, RecursiveMode::NonRecursive);
                            }
                        }
                        current_watched_shader = resolved_path.clone();
                    }

                    {
                        let mut path_guard = state.config.resolved_shader_path.write().await;
                        *path_guard = resolved_path;
                    }

                    let payload = json!({
                        "type": "CONFIG_UPDATE",
                        "config": {
                            "thumbnail_size": thumb_size,
                            "shader": shader_cfg
                        }
                    })
                    .to_string();

                    let _ = state.tx.send(payload);
                }
                Err(e) => {
                    log::error!("Failed to reload config: {}", e);
                }
            }
        }
    });
}
