use crate::server::state::UpdateScope;
use crate::update::cache::{get_lua_config_hash, FileStat};
use crate::update::verify::{find_missing_paths, verify_albums_parallel};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_work_queue(
    scope: &UpdateScope,
    music_directory: &Path,
    cache: &HashMap<String, FileStat>,
    force: bool,
    effective_jobs: Option<usize>,
    tracked: Option<(Vec<PathBuf>, Vec<PathBuf>)>,
    silent: bool,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    if !force && let Some((wq, mp)) = tracked {
        if !silent && !wq.is_empty() {
            log::info!("Verifying {} tracked albums...", wq.len());
        }

        let results = verify_albums_parallel(wq, cache, force, effective_jobs, music_directory)?;
        let mut verified_wq = Vec::new();
        for (path, is_dirty) in results {
            if is_dirty {
                verified_wq.push(path);
            }
        }
        Ok((verified_wq, mp))
    } else {
        let (all_albums, mp) = match scope {
            UpdateScope::All => {
                let albums = libdale::scanner::find_target_albums(music_directory)?;
                let missing = find_missing_paths(&albums, music_directory, music_directory, cache);
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
                    let missing = find_missing_paths(target_albums, music_directory, &p_canon, cache);
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

        if !silent && !all_albums.is_empty() {
            log::info!("Verifying {} albums...", all_albums.len());
        }

        let results = verify_albums_parallel(all_albums, cache, force, effective_jobs, music_directory)?;
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
    work_queue: &[PathBuf],
    missing_paths: &[PathBuf],
    music_directory: &Path,
    silent: bool,
) {
    for album_root in work_queue {
        let album_id = libdale::resolvers::rel_path(album_root, music_directory);
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

        for entry in walkdir::WalkDir::new(album_root)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            let p = entry.path();
            if p.is_file() {
                let rel = libdale::resolvers::rel_path(p, music_directory);
                if let Ok(m) = entry.metadata() {
                    let mtime = m
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let size = m.len();
                    cache.insert(rel, FileStat { mtime, size });
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
