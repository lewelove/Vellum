use crate::config::AppConfig;
use crate::server::state::AppState;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub fn start(config_path: PathBuf, state: Arc<AppState>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() {
                        let _ = tx.blocking_send(());
                    }
                }
            },
            notify::Config::default(),
        )
        .expect("Failed to create config watcher");

        watcher
            .watch(&config_path, RecursiveMode::NonRecursive)
            .expect("Failed to watch config file");

        while rx.recv().await.is_some() {
            tokio::time::sleep(Duration::from_millis(100)).await;
            while rx.try_recv().is_ok() {}

            log::info!("Config change detected, reloading...");

            match AppConfig::load() {
                Ok((new_config, _, _)) => {
                    let thumb_size = new_config.theme.as_ref().map_or(200, |t| t.thumbnail_size);
                    let shader = new_config.theme.as_ref().and_then(|t| t.shader.clone());

                    let payload = json!({
                        "type": "CONFIG_UPDATE",
                        "config": {
                            "thumbnail_size": thumb_size,
                            "shader": shader
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
