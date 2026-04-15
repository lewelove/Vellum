use crate::config::AppConfig;
use crate::server::state::AppState;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub fn start(config_path: PathBuf, state: Arc<AppState>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<PathBuf>>(10);
    let config_dir = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();

    tokio::spawn(async move {
        let tx_clone = tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        let _ = tx_clone.blocking_send(event.paths);
                    }
                }
            },
            notify::Config::default(),
        )
        .expect("Failed to create config watcher");

        watcher
            .watch(&config_dir, RecursiveMode::NonRecursive)
            .expect("Failed to watch config directory");

        let mut current_watched_shader: Option<PathBuf> = None;

        let initial_shader_path = {
            let guard = state.config.read().await;
            guard.resolved_shader_path.clone()
        };

        if let Some(ref p) = initial_shader_path {
            if p.exists() && !p.starts_with(&config_dir) {
                let _ = watcher.watch(p, RecursiveMode::NonRecursive);
                current_watched_shader = Some(p.clone());
            }
        }

        while let Some(mut paths) = rx.recv().await {
            tokio::time::sleep(Duration::from_millis(100)).await;
            while let Ok(more_paths) = rx.try_recv() {
                paths.extend(more_paths);
            }

            let mut config_changed = false;
            let mut css_changed = false;
            let mut logic_changed = false;

            for p in &paths {
                if *p == config_path {
                    config_changed = true;
                }
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    match name {
                        "vellum.css" => {
                            css_changed = true;
                            let mut guard = state.config.write().await;
                            guard.resolved_css_path = if p.exists() { Some(p.clone()) } else { None };
                        }
                        "logic.json" => {
                            logic_changed = true;
                            let mut guard = state.config.write().await;
                            guard.resolved_logic_path = if p.exists() { Some(p.clone()) } else { None };
                        }
                        _ => {}
                    }
                }
                if let Some(ref sp) = current_watched_shader {
                    if *p == *sp {
                        config_changed = true;
                    }
                }
            }

            if css_changed {
                log::info!("Filesystem change: reloading custom CSS...");
                let payload = json!({ "type": "THEME_UPDATE" }).to_string();
                let _ = state.tx.send(payload);
            }

            if logic_changed {
                log::info!("Filesystem change: reloading logic.json...");
                if let Some(ref logic_path) = state.config.read().await.resolved_logic_path {
                    let mut query = state.query.lock().await;
                    if let Err(e) = query.reload_manifest(logic_path) {
                        log::error!("Failed to reload logic.json: {}", e);
                    }
                }
                let payload = json!({ "type": "LOGIC_UPDATE" }).to_string();
                let _ = state.tx.send(payload);
            }

            if config_changed {
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
                                if new_p.exists() && !new_p.starts_with(&config_dir) {
                                    let _ = watcher.watch(new_p, RecursiveMode::NonRecursive);
                                }
                            }
                            current_watched_shader = resolved_path.clone();
                        }

                        {
                            let mut config_guard = state.config.write().await;
                            config_guard.thumbnail_size = thumb_size;
                            config_guard.shader = shader_cfg.clone();
                            config_guard.resolved_shader_path = resolved_path.clone();
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
        }
    });
}
