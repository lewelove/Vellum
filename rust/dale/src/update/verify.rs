use crate::update::cache::FileStat;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn find_missing_paths(
    all_albums: &[PathBuf],
    library_root: &Path,
    scan_root: &Path,
    cache: &HashMap<String, FileStat>,
) -> Vec<PathBuf> {
    let mut missing_paths = Vec::new();
    let current_album_ids: HashSet<String> = all_albums
        .iter()
        .map(|p| libdale::resolvers::rel_path(p, library_root))
        .collect();

    let mut cached_album_ids = HashSet::new();
    for rel_path in cache.keys() {
        if rel_path == "metadata.toml" {
            cached_album_ids.insert(String::new());
        } else if let Some(album_id) = rel_path.strip_suffix("/metadata.toml") {
            cached_album_ids.insert(album_id.to_string());
        }
    }

    for cached_id in cached_album_ids {
        let full_cached_path = library_root.join(&cached_id);
        if full_cached_path.starts_with(scan_root) && !current_album_ids.contains(&cached_id) {
            missing_paths.push(full_cached_path);
        }
    }
    missing_paths
}

pub fn verify_albums_parallel(
    albums: Vec<PathBuf>,
    cache: &HashMap<String, FileStat>,
    force: bool,
    jobs: Option<usize>,
    library_root: &Path,
) -> Result<Vec<(PathBuf, bool)>> {
    let default_parallelism = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs.unwrap_or(default_parallelism))
        .build()?;

    Ok(pool.install(|| {
        albums
            .into_par_iter()
            .map(|album_root| {
                let is_dirty = force || is_album_dirty(&album_root, cache, library_root);
                (album_root, is_dirty)
            })
            .collect()
    }))
}

fn is_album_dirty(
    album_root: &Path,
    cache: &HashMap<String, FileStat>,
    library_root: &Path,
) -> bool {
    let lock_path = album_root.join("album.lock.json");
    if !lock_path.exists() {
        return true;
    }

    let meta_path = album_root.join("metadata.toml");
    if !meta_path.exists() {
        return true;
    }

    let album_id = libdale::resolvers::rel_path(album_root, library_root);
    let prefix = if album_id.is_empty() {
        String::new()
    } else {
        format!("{album_id}/")
    };

    deep_verify_dirty(album_root, cache, &prefix, library_root)
}

fn deep_verify_dirty(
    album_root: &Path,
    cache: &HashMap<String, FileStat>,
    prefix: &str,
    library_root: &Path,
) -> bool {
    let mut album_files_changed = false;
    let mut seen_paths = HashSet::new();

    for entry in walkdir::WalkDir::new(album_root)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.is_file() {
            let rel = libdale::resolvers::rel_path(p, library_root);
            let Ok(m) = entry.metadata() else {
                return true;
            };
            let mtime = m
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let size = m.len();

            let cached_stat = cache.get(&rel);
            if cached_stat != Some(&FileStat { mtime, size }) {
                album_files_changed = true;
                break;
            }
            seen_paths.insert(rel);
        }
    }

    if !album_files_changed {
        for k in cache.keys() {
            let matches_album = if prefix.is_empty() {
                !k.contains('/')
            } else {
                k.starts_with(prefix)
            };
            if matches_album && !seen_paths.contains(k) {
                return true;
            }
        }
    }

    album_files_changed
}
