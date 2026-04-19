use crate::config::AppConfig;
use crate::server::state::AppState;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub fn start(config_path: PathBuf, state: Arc<AppState>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<PathBuf>>(10);
    
    let canon_config_path = config_path.canonicalize().unwrap_or_else(|_| config_path.clone());
    let config_dir = canon_config_path.parent().unwrap_or(Path::new(".")).to_path_buf();

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
            .watch(&config_dir, RecursiveMode::Recursive)
            .expect("Failed to watch config directory");

        let mut current_watched_shader: Option<PathBuf> = {
            let guard = state.config.read().await;
            guard.resolved_shader_path.clone()
        };

        if let Some(ref p) = current_watched_shader {
            if p.exists() && !p.starts_with(&config_dir) {
                let _ = watcher.watch(p, RecursiveMode::NonRecursive);
            }
        }

        let mut current_watched_shelf_files: Vec<PathBuf> = {
            let guard = state.config.read().await;
            guard.resolved_shelf_files.clone()
        };

        for p in &current_watched_shelf_files {
            if p.exists() && !p.starts_with(&config_dir) {
                let _ = watcher.watch(p, RecursiveMode::NonRecursive);
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
            let mut shelf_changed = false;

            for p in paths {
                let p = p.canonicalize().unwrap_or(p);

                if p == canon_config_path {
                    config_changed = true;
                }

                if let Some(ref sp) = current_watched_shader {
                    if p == *sp {
                        config_changed = true;
                    }
                }

                if current_watched_shelf_files.contains(&p) {
                    shelf_changed = true;
                }

                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    match name {
                        "vellum.css" => {
                            css_changed = true;
                        }
                        "logic.toml" => {
                            logic_changed = true;
                        }
                        _ => {}
                    }
                }
            }

            if css_changed {
                log::info!("Filesystem change: reloading custom CSS...");
                let mut guard = state.config.write().await;
                let css_path = config_dir.join("vellum.css");
                guard.resolved_css_path = if css_path.exists() { css_path.canonicalize().ok() } else { None };
                
                let payload = json!({ "type": "THEME_UPDATE" }).to_string();
                let _ = state.tx.send(payload);
            }

            if logic_changed {
                log::info!("Filesystem change: reloading logic.toml...");
                let logic_path = config_dir.join("logic.toml");
                let resolved = if logic_path.exists() { logic_path.canonicalize().ok() } else { None };
                
                let mut new_shelf_files = Vec::new();

                {
                    let mut guard = state.config.write().await;
                    guard.resolved_logic_path = resolved.clone();
                }

                if let Some(ref lp) = resolved {
                    let mut query = state.query.lock().await;
                    if let Err(e) = query.reload_manifest(lp) {
                        log::error!("Failed to reload logic.toml: {}", e);
                    } else {
                        for shelf in query.manifest.shelves.values() {
                            if let Some(file) = &shelf.file {
                                let expanded = crate::expand_path(file);
                                new_shelf_files.push(expanded.canonicalize().unwrap_or(expanded));
                            }
                        }
                    }
                }

                for p in &current_watched_shelf_files {
                    if !new_shelf_files.contains(p) && !p.starts_with(&config_dir) {
                        let _ = watcher.unwatch(p);
                    }
                }
                for p in &new_shelf_files {
                    if !current_watched_shelf_files.contains(p) && p.exists() && !p.starts_with(&config_dir) {
                        let _ = watcher.watch(p, RecursiveMode::NonRecursive);
                    }
                }
                current_watched_shelf_files = new_shelf_files.clone();
                
                {
                    let mut guard = state.config.write().await;
                    guard.resolved_shelf_files = new_shelf_files;
                }

                let payload = json!({ "type": "LOGIC_UPDATE" }).to_string();
                let _ = state.tx.send(payload);
            }

            if shelf_changed && !logic_changed {
                log::info!("Filesystem change: reloading shelf files...");
                let mut query = state.query.lock().await;
                if let Err(e) = query.build_cache() {
                    log::error!("Failed to rebuild query cache: {}", e);
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

                        let mut next_shader_path = None;
                        if let Some(ref s) = shader_cfg 
                            && let Some(ref p) = s.path 
                        {
                            let expanded = crate::expand_path(p);
                            let absolute = if expanded.is_absolute() {
                                expanded
                            } else {
                                config_dir.join(expanded)
                            };
                            next_shader_path = absolute.canonicalize().ok().or(Some(absolute));
                        }

                        if next_shader_path != current_watched_shader {
                            if let Some(ref old_p) = current_watched_shader {
                                if !old_p.starts_with(&config_dir) {
                                    let _ = watcher.unwatch(old_p);
                                }
                            }
                            if let Some(ref new_p) = next_shader_path {
                                if new_p.exists() && !new_p.starts_with(&config_dir) {
                                    let _ = watcher.watch(new_p, RecursiveMode::NonRecursive);
                                }
                            }
                            current_watched_shader = next_shader_path.clone();
                        }

                        {
                            let mut config_guard = state.config.write().await;
                            config_guard.thumbnail_size = thumb_size;
                            config_guard.shader = shader_cfg.clone();
                            config_guard.resolved_shader_path = next_shader_path.clone();
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
