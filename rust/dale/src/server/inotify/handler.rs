use crate::server::inotify::classifier::ChangeFlags;
use crate::server::state::AppState;
use rayon::prelude::*;
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
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
        handle_album_changes(flags.changed_albums, state).await;
    }
}

async fn handle_album_changes(changed_albums: HashSet<PathBuf>, state: &Arc<AppState>) {
    let config_path = libdale::lua::resolve_config_path().unwrap_or_default();
    let music_directory = state.config.read().await.music_directory.clone();

    let resolved_lua_config =
        if let Ok(Ok(cfg)) = tokio::task::spawn_blocking(libdale::lua::ResolvedConfig::load).await
        {
            Arc::new(cfg)
        } else {
            log::error!("Failed to load resolved config for album update");
            return;
        };

    let active_writes = Arc::clone(&state.active_writes);

    let recompiled_items = tokio::task::spawn_blocking(move || {
        let music_dir_ref = &music_directory;
        let config_ref = &resolved_lua_config;
        let config_path_ref = &config_path;

        changed_albums
            .into_par_iter()
            .map_init(
                || match libdale::lua::LuaEngine::new() {
                    Ok(engine) => match engine.evaluate_config(config_path_ref) {
                        Ok(_) => Some(engine),
                        Err(e) => {
                            log::error!("Failed to evaluate config in inotify handler: {e}");
                            None
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to initialize Lua engine in inotify handler: {e}");
                        None
                    }
                },
                |engine_opt, album_path| {
                    let rel = album_path.strip_prefix(music_dir_ref).unwrap_or(&album_path);
                    let album_id = rel.to_string_lossy().to_string();

                    if !album_path.exists() {
                        return Some((album_id, String::new(), None, HashSet::new()));
                    }

                    let lock_file_path = album_path.join("album.lock.json");

                    let Some(engine) = engine_opt.as_ref() else {
                        log::error!("Skipping recompilation of {album_id} due to Lua engine init failure");
                        return None;
                    };

                    let Ok(res) =
                        crate::compile::build::build(&album_path, config_ref, engine)
                    else {
                        log::warn!("Compilation failed for {album_id}");
                        return None;
                    };

                    let mut lock_val = res.lock_json;
                    let _ = lock_val.as_object_mut().and_then(|o| o.remove("ctx"));
                    crate::compile::utils::strip_empty_values(&mut lock_val);

                    let Ok(lock_json) = serde_json::to_string_pretty(&lock_val) else {
                        return None;
                    };

                    let eval_res = engine.evaluate_album_logic(&lock_val).ok();

                    let should_write = std::fs::read_to_string(&lock_file_path)
                        .map_or(true, |existing| existing != lock_json);

                    if should_write {
                        if let Ok(mut active) = active_writes.lock() {
                            if let Ok(canon) = lock_file_path.canonicalize() {
                                active.insert(canon);
                            }
                            active.insert(lock_file_path.clone());
                        }
                        let _ = std::fs::write(&lock_file_path, &lock_json);
                    }

                    Some((album_id, lock_json, eval_res, res.dependencies))
                },
            )
            .flatten()
            .collect::<Vec<_>>()
    })
    .await
    .unwrap_or_default();

    if !recompiled_items.is_empty() {
        ingest_and_broadcast_albums(recompiled_items, false, state).await;
    }
}

pub async fn ingest_and_broadcast_albums(
    recompiled_items: Vec<(String, String, Option<serde_json::Value>, HashSet<PathBuf>)>,
    is_internal_update: bool,
    state: &Arc<AppState>,
) {
    let music_directory = state.config.read().await.music_directory.clone();

    {
        let mut deps_graph = state.deps_graph.write().await;
        for (album_id, _, eval_res, dependencies) in &recompiled_items {
            let album_path = music_directory.join(album_id);
            let album_path_canon = album_path.canonicalize().unwrap_or(album_path);
            if eval_res.is_some() {
                deps_graph.update_album_deps(album_path_canon, dependencies.clone());
            } else {
                deps_graph.remove_album(&album_path_canon);
            }
        }
    }

    let (updated_dict_entries, removed_ids, shelves) = {
        let mut logic = state.logic.write().await;
        let mut entries = std::collections::HashMap::new();
        let mut removed = Vec::new();

        for (album_id, lock_json, eval_res, _) in recompiled_items {
            logic.remove_album(&album_id);
            if let Some(eval) = eval_res {
                if !is_internal_update {
                    log::info!("Updated album state: {album_id}");
                }

                let _ = logic.ingest_pre_evaluated(&album_id, &lock_json, eval);
                if let Some(entry) = logic.dict.get(&album_id).cloned() {
                    entries.insert(album_id, entry);
                }
            } else {
                log::info!("Removed album state: {album_id}");
                removed.push(album_id);
            }
        }

        logic.build_cache();

        let mut s = std::collections::HashMap::new();
        for key in logic.manifest.shelves.keys() {
            s.insert(key.clone(), logic.request_shelf_view(key, None, false));
        }
        drop(logic);
        (entries, removed, s)
    };

    if !updated_dict_entries.is_empty() || !removed_ids.is_empty() {
        let _ = state.tx.send(
            json!({
                "type": "ALBUMS_UPDATED",
                "updated": updated_dict_entries,
                "removed": removed_ids,
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
