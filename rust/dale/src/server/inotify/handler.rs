use crate::server::inotify::classifier::ChangeFlags;
use crate::server::state::AppState;
use serde_json::json;
use std::sync::Arc;

pub async fn process_events(flags: ChangeFlags, state: &Arc<AppState>) {
    for intf_name in flags.interfaces_asset {
        log::info!("Interface '{intf_name}' asset changed.");
        let _ = state.tx.send(
            json!({
                "type": "INTERFACE_ASSET_UPDATE",
                "name": intf_name
            })
            .to_string(),
        );
    }

    if flags.config {
        handle_config_change(state).await;
    }

    if !flags.changed_albums.is_empty() {
        handle_album_manifest_changes(&flags.changed_albums, state).await;
    }
}

async fn handle_album_manifest_changes(
    changed_albums: &std::collections::HashSet<std::path::PathBuf>,
    state: &Arc<AppState>,
) {
    let config_path = libdale::lua::resolve_config_path().unwrap_or_default();

    let resolved_lua_config =
        if let Ok(Ok(cfg)) = tokio::task::spawn_blocking(libdale::lua::ResolvedConfig::load).await
        {
            Arc::new(cfg)
        } else {
            log::error!("Failed to load resolved config for album compile");
            return;
        };

    let mut recompiled_items = Vec::new();

    for album_path in changed_albums {
        if let Some(item) =
            recompile_single_album(album_path, &resolved_lua_config, &config_path).await
        {
            recompiled_items.push(item);
        }
    }

    if !recompiled_items.is_empty() {
        ingest_and_broadcast_albums(recompiled_items, state).await;
    }
}

async fn recompile_single_album(
    album_path: &std::path::Path,
    resolved_lua_config: &Arc<libdale::lua::ResolvedConfig>,
    config_path: &std::path::Path,
) -> Option<(String, String, Option<serde_json::Value>)> {
    let album_path_clone = album_path.to_path_buf();
    let cfg = Arc::clone(resolved_lua_config);
    let c_path = config_path.to_path_buf();

    let process_res = tokio::task::spawn_blocking(
        move || -> Result<(String, String, Option<serde_json::Value>), anyhow::Error> {
            let mut lock_val = crate::compile::build::build(&album_path_clone, &cfg)?;

            let _ = lock_val.as_object_mut().and_then(|o| o.remove("ctx"));
            crate::compile::utils::strip_empty_values(&mut lock_val);

            let lock_json = serde_json::to_string_pretty(&lock_val)?;

            let eval_res = libdale::lua::get_or_init_lua_vm(&c_path, |engine| {
                let parsed: serde_json::Value =
                    serde_json::from_str(&lock_json).unwrap_or_default();
                engine.evaluate_album_logic(&parsed)
            })
            .ok();

            let rel_path = album_path_clone
                .strip_prefix(&cfg.app.storage.music_directory)
                .unwrap_or(&album_path_clone);
            let album_id = rel_path.to_string_lossy().to_string();

            Ok((album_id, lock_json, eval_res))
        },
    )
    .await;

    match process_res {
        Ok(Ok((album_id, lock_json, eval_res))) => {
            let lock_file_path = album_path.join("album.lock.json");
            let should_write = tokio::fs::read_to_string(&lock_file_path)
                .await
                .map_or(true, |existing| existing != lock_json);

            if should_write
                && let Err(e) = tokio::fs::write(&lock_file_path, &lock_json).await
            {
                log::error!(
                    "Failed to write lock file at {}: {e}",
                    lock_file_path.display()
                );
                return None;
            }

            Some((album_id, lock_json, eval_res))
        }
        Ok(Err(e)) => {
            log::error!(
                "Failed to recompile album manifest at {}: {e}",
                album_path.display()
            );
            None
        }
        Err(e) => {
            log::error!(
                "Task panicked while compiling {}: {e}",
                album_path.display()
            );
            None
        }
    }
}

async fn ingest_and_broadcast_albums(
    recompiled_items: Vec<(String, String, Option<serde_json::Value>)>,
    state: &Arc<AppState>,
) {
    let (updated_dict_entries, shelves) = {
        let mut logic = state.logic.write().await;
        let mut entries = std::collections::HashMap::new();

        for (album_id, lock_json, eval_res) in recompiled_items {
            logic.remove_album(&album_id);
            if let Some(eval) = eval_res {
                let _ = logic.ingest_pre_evaluated(&album_id, &lock_json, eval);
            } else {
                let _ = logic.ingest(&album_id, &lock_json);
            }
            log::info!("Hot reloaded manifest for album: {album_id}");
            if let Some(entry) = logic.dict.get(&album_id).cloned() {
                entries.insert(album_id, entry);
            }
        }

        logic.build_cache();

        let mut s = std::collections::HashMap::new();
        for key in logic.manifest.shelves.keys() {
            s.insert(key.clone(), logic.request_shelf_view(key, None, false));
        }
        drop(logic);
        (entries, s)
    };

    for (album_id, dict_entry) in updated_dict_entries {
        let _ = state.tx.send(
            json!({
                "type": "ALBUM_UPDATED",
                "id": album_id,
                "dictEntry": dict_entry,
                "shelves": shelves
            })
            .to_string(),
        );
    }
}

async fn handle_config_change(state: &Arc<AppState>) {
    log::info!("Filesystem change: reloading config and logic...");

    match libdale::lua::ResolvedConfig::load() {
        Ok(new_config) => {
            let covers = new_config.covers.clone();
            let new_interfaces = new_config.interfaces.clone();
            let new_actions = new_config.actions.clone();
            let dependencies = new_config.dependencies.clone();
            let config_path = new_config.path.clone();

            {
                let mut config_guard = state.config.write().await;
                config_guard.covers.clone_from(&covers);
                config_guard.interfaces.clone_from(&new_interfaces);
                config_guard.actions.clone_from(&new_actions);
                config_guard
                    .resolved_dependencies
                    .clone_from(&dependencies);
            }

            let manifest = {
                let mut logic = state.logic.write().await;
                if let Err(e) = logic.reload_manifest(&config_path) {
                    log::error!("Failed to reload logic manifest: {e}");
                }
                logic.manifest.clone()
            };

            let _ = state.tx.send(
                json!({
                    "type": "CONFIG_UPDATE",
                    "config": {
                        "covers": covers
                    }
                })
                .to_string(),
            );

            let _ = state.tx.send(
                json!({
                    "type": "LOGIC_UPDATE",
                    "manifest": manifest
                })
                .to_string(),
            );

            for (name, cfg) in &new_interfaces {
                let _ = state.tx.send(
                    json!({
                        "type": "INTERFACE_CONFIG_UPDATE",
                        "name": name,
                        "config": cfg.config
                    })
                    .to_string(),
                );
            }
        }
        Err(e) => {
            log::error!("Failed to reload config: {e:?}");
        }
    }
}
