use crate::server::inotify::classifier::ChangeFlags;
use crate::server::state::AppState;
use rayon::prelude::*;
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

pub type RecompiledAlbumItem = (
    PathBuf,
    String,
    String,
    Option<serde_json::Value>,
    HashSet<PathBuf>,
);

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

    if !flags.vanished_paths.is_empty() || !flags.changed_albums.is_empty() {
        handle_album_changes(flags.vanished_paths, flags.changed_albums, state).await;
    }
}

async fn handle_album_changes(
    vanished_paths: HashSet<PathBuf>,
    changed_albums: HashSet<PathBuf>,
    state: &Arc<AppState>,
) {
    let config_path = libdale::lua::resolve_config_path().unwrap_or_default();

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
        let config_ref = &resolved_lua_config;
        let config_path_ref = &config_path;

        changed_albums
            .into_par_iter()
            .filter(|p| p.exists())
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
                    let lock_file_path = album_path.join("album.lock.json");

                    let Some(engine) = engine_opt.as_ref() else {
                        log::error!(
                            "Skipping recompilation of {} due to Lua engine init failure",
                            album_path.display()
                        );
                        return None;
                    };

                    let Ok(res) =
                        crate::compile::build::build(&album_path, config_ref, engine)
                    else {
                        log::warn!("Compilation failed for {}", album_path.display());
                        return None;
                    };

                    let Ok(lock_json) = serde_json::to_string_pretty(&res.lock_json) else {
                        return None;
                    };

                    let eval_res = engine.evaluate_album_logic(&res.lock_json).ok();

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

                    Some((res.album_dir, res.album_id, lock_json, eval_res, res.dependencies))
                },
            )
            .flatten()
            .collect::<Vec<_>>()
    })
    .await
    .unwrap_or_default();

    ingest_and_broadcast_albums(vanished_paths, recompiled_items, false, state).await;
}

pub async fn ingest_and_broadcast_albums(
    vanished_paths: HashSet<PathBuf>,
    recompiled_items: Vec<RecompiledAlbumItem>,
    is_internal_update: bool,
    state: &Arc<AppState>,
) {
    let music_directory = state.config.read().await.music_directory.clone();

    let mut removed_ids = Vec::new();
    let mut updated_dict_entries = std::collections::HashMap::new();

    {
        let mut logic = state.logic.write().await;
        let mut deps_graph = state.deps_graph.write().await;

        for vanished in &vanished_paths {
            let vanished_resolved = vanished.parent().map_or_else(
                || vanished.clone(),
                |parent| {
                    parent.canonicalize().map_or_else(
                        |_| vanished.clone(),
                        |parent_canon| parent_canon.join(vanished.file_name().unwrap_or_default()),
                    )
                },
            );

            let dead_paths: Vec<PathBuf> = logic
                .albums_by_path
                .keys()
                .filter(|path| {
                    path.starts_with(vanished)
                        || *path == vanished
                        || path.starts_with(&vanished_resolved)
                        || *path == &vanished_resolved
                        || (vanished.starts_with(path) && !path.join("metadata.toml").exists())
                        || (vanished_resolved.starts_with(path) && !path.join("metadata.toml").exists())
                })
                .cloned()
                .collect();

            for dead_path in dead_paths {
                if let Some(dead_id) = logic.remove_album_by_path(&dead_path) {
                    if !is_internal_update {
                        log::info!("Removed album: {dead_id}");
                    }
                    deps_graph.remove_album(&dead_path);
                    removed_ids.push(dead_id);
                }
            }
        }

        for (album_path, album_id, lock_json, eval_res, dependencies) in recompiled_items {
            let album_path_canon = album_path.canonicalize().unwrap_or_else(|_| album_path.clone());

            if let Some(eval) = eval_res {
                match logic.ingest_pre_evaluated(
                    &album_path_canon,
                    &album_id,
                    &lock_json,
                    eval,
                    &music_directory,
                ) {
                    Ok(()) => {
                        if !is_internal_update {
                            log::info!("Updated album: {album_id}");
                        }
                        deps_graph.update_album_deps(album_path_canon, dependencies);
                        if let Some(entry) = logic.dict.get(&album_id).cloned() {
                            updated_dict_entries.insert(album_id, entry);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to ingest album at {}: {e}", album_path.display());
                    }
                }
            } else if !album_path_canon.exists()
                && let Some(dead_id) = logic.remove_album_by_path(&album_path_canon)
            {
                if !is_internal_update {
                    log::info!("Removed album: {dead_id}");
                }
                deps_graph.remove_album(&album_path_canon);
                removed_ids.push(dead_id);
            }
        }

        drop(deps_graph);
        logic.build_cache();
    }

    let shelves = {
        let logic = state.logic.read().await;
        let mut s = std::collections::HashMap::new();
        for key in logic.manifest.shelves.keys() {
            s.insert(key.clone(), logic.request_shelf_view(key, None, false));
        }
        s
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
