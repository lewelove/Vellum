use crate::server::state::{DependencyGraph, UpdateScope};
use crate::update::cache::{get_lua_config_hash, FileStat};
use crate::update::verify::{find_missing_paths, verify_albums_parallel};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub struct WorkQueueContext<'a> {
    pub scope: &'a UpdateScope,
    pub music_directory: &'a Path,
    pub cache: &'a HashMap<String, FileStat>,
    pub deps_graph: &'a DependencyGraph,
    pub force: bool,
    pub effective_jobs: Option<usize>,
    pub tracked: Option<(Vec<PathBuf>, Vec<PathBuf>)>,
    pub silent: bool,
}

pub fn resolve_work_queue(
    ctx: WorkQueueContext<'_>,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    if !ctx.force && let Some((wq, mp)) = ctx.tracked {
        if !ctx.silent && !wq.is_empty() {
            log::info!("Verifying {} tracked albums...", wq.len());
        }

        let results = verify_albums_parallel(
            wq,
            ctx.cache,
            ctx.deps_graph,
            ctx.force,
            ctx.effective_jobs,
            ctx.music_directory,
        )?;
        let mut verified_wq = Vec::new();
        for (path, is_dirty) in results {
            if is_dirty {
                verified_wq.push(path);
            }
        }
        Ok((verified_wq, mp))
    } else {
        let (all_albums, mp) = match ctx.scope {
            UpdateScope::All => {
                let albums = libdale::scanner::find_target_albums(ctx.music_directory)?;
                let missing = find_missing_paths(&albums, ctx.music_directory, ctx.music_directory, ctx.cache);
                (albums, missing)
            }
            UpdateScope::Paths(paths) => {
                let mut albums_set = HashSet::new();
                let mut missing_set = HashSet::new();

                for p in paths {
                    let p_canon = p.canonicalize().unwrap_or_else(|_| p.clone());
                    if let Ok(found) = libdale::scanner::find_target_albums(&p_canon) {
                        for album in found {
                            albums_set.insert(album);
                        }
                    }
                }

                let all_found: Vec<PathBuf> = albums_set.iter().cloned().collect();

                for p in paths {
                    let p_canon = p.canonicalize().unwrap_or_else(|_| p.clone());
                    let target_albums = if p_canon.exists() {
                        &all_found[..]
                    } else {
                        &[][..]
                    };
                    let missing = find_missing_paths(target_albums, ctx.music_directory, &p_canon, ctx.cache);
                    for m in missing {
                        missing_set.insert(m);
                    }
                }

                let mut albums_vec: Vec<PathBuf> = albums_set.into_iter().collect();
                albums_vec.sort();
                let mut missing_vec: Vec<PathBuf> = missing_set.into_iter().collect();
                missing_vec.sort();
                (albums_vec, missing_vec)
            }
        };

        if !ctx.silent && !all_albums.is_empty() {
            log::info!("Verifying {} albums...", all_albums.len());
        }

        let results = verify_albums_parallel(
            all_albums,
            ctx.cache,
            ctx.deps_graph,
            ctx.force,
            ctx.effective_jobs,
            ctx.music_directory,
        )?;
        let mut wq = Vec::new();
        for (path, is_dirty) in results {
            if is_dirty {
                wq.push(path);
            }
        }
        Ok((wq, mp))
    }
}

pub fn update_cache_entries(
    cache: &mut HashMap<String, FileStat>,
    deps_graph: &DependencyGraph,
    work_queue: &[PathBuf],
    missing_paths: &[PathBuf],
    music_directory: &Path,
    silent: bool,
) {
    for album_root in work_queue {
        let album_root_canon = album_root
            .canonicalize()
            .unwrap_or_else(|_| album_root.clone());
        let album_id = libdale::resolvers::rel_path(&album_root_canon, music_directory);
        let prefix = if album_id.is_empty() {
            String::new()
        } else {
            format!("{album_id}/")
        };

        cache.retain(|k, _| {
            if prefix.is_empty() {
                k.contains('/')
            } else {
                !k.starts_with(&prefix)
            }
        });

        for entry in walkdir::WalkDir::new(&album_root_canon)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            let p = entry.path();
            if p.is_file() {
                let rel = libdale::resolvers::rel_path(p, music_directory);
                if let Ok(m) = entry.metadata() {
                    cache.insert(rel, FileStat::from(&m));
                }
            }
        }

        if let Some(external_deps) = deps_graph.album_to_files.get(&album_root_canon) {
            for dep_path in external_deps {
                if !dep_path.starts_with(&album_root_canon)
                    && let Ok(m) = dep_path.metadata()
                {
                    let rel = libdale::resolvers::rel_path(dep_path, music_directory);
                    cache.insert(rel, FileStat::from(&m));
                }
            }
        }
    }

    for missing in missing_paths {
        let album_id = libdale::resolvers::rel_path(missing, music_directory);
        let prefix = format!("{album_id}/");

        if !silent {
            let display_path = missing.strip_prefix(music_directory).unwrap_or(missing);
            log::info!("Removed: {}", display_path.display());
        }

        cache.retain(|k, _| !k.starts_with(&prefix));
    }
}

pub async fn try_get_server_tracked_albums(
    music_directory: &Path,
) -> Option<(Vec<PathBuf>, Vec<PathBuf>)> {
    let resp = reqwest::Client::new()
        .get("http://127.0.0.1:8000/api/internal/tracked_albums")
        .timeout(std::time::Duration::from_millis(500))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let data: serde_json::Value = resp.json().await.ok()?;
    if data
        .get("full_rescan")
        .and_then(serde_json::Value::as_bool)
        != Some(false)
    {
        return None;
    }

    let tracked_arr = data.get("tracked_albums")?.as_array()?;
    let mut work_queue = Vec::new();
    let mut missing_paths = Vec::new();

    for item in tracked_arr {
        if let Some(p_str) = item.as_str() {
            let p = if Path::new(p_str).is_absolute() {
                PathBuf::from(p_str)
            } else {
                music_directory.join(p_str)
            };
            if p.exists() {
                work_queue.push(p);
            } else {
                missing_paths.push(p);
            }
        }
    }

    Some((work_queue, missing_paths))
}

pub fn check_lua_config_changed(
    dependencies: &[PathBuf],
    cache_root: &Path,
    silent: bool,
) -> (bool, PathBuf, String) {
    let lua_hash = get_lua_config_hash(dependencies);
    let lua_hash_file = cache_root.join("config.blake3");
    let previous_lua_hash = fs::read_to_string(&lua_hash_file).unwrap_or_default();
    let config_changed = lua_hash != previous_lua_hash;

    if config_changed && !silent {
        log::info!("Lua configuration changed. Re-evaluating album locks...");
    }

    (config_changed, lua_hash_file, lua_hash)
}
