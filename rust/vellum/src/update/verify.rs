use crate::update::cache::{AlbumCacheEntry, get_mtime_sum};
use anyhow::Result;
use libvellum::error::VellumError;
use libvellum::sentinel::{TrustState, verify_trust};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn find_missing_paths(
    all_albums: &[PathBuf],
    scan_root: &Path,
    cache: &HashMap<String, AlbumCacheEntry>,
) -> Vec<PathBuf> {
    let mut missing_paths = Vec::new();
    let album_set: HashSet<PathBuf> = all_albums.iter().cloned().collect();
    let scan_root_canon = scan_root.canonicalize().unwrap_or_else(|_| scan_root.to_path_buf());

    for cached_path_str in cache.keys() {
        let cached_path = PathBuf::from(cached_path_str);
        if cached_path.starts_with(&scan_root_canon) && !album_set.contains(&cached_path) {
            missing_paths.push(cached_path);
        }
    }
    missing_paths
}

pub fn verify_albums_parallel(
    albums: Vec<PathBuf>,
    cache: &HashMap<String, AlbumCacheEntry>,
    force: bool,
    jobs: Option<usize>,
    exts: &[String],
    manifests: Option<&Vec<String>>,
    library_root: &Path,
) -> Result<Vec<(PathBuf, u64, bool)>> {
    let default_parallelism = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs.unwrap_or(default_parallelism))
        .build()?;

    Ok(pool.install(|| {
        albums
            .into_par_iter()
            .map(|album_root| {
                let album_path_str = album_root.to_string_lossy().to_string();
                let metadata_path = album_root.join("metadata.toml");
                let mtime_sum = get_mtime_sum(&album_root, &metadata_path, exts, manifests);

                if force {
                    return (album_root, mtime_sum, true);
                }

                let expected_id = libvellum::resolvers::rel_path(&album_root, library_root);

                if let Some(entry) = cache.get(&album_path_str)
                    && entry.mtime_sum == mtime_sum && mtime_sum != 0
                {
                    match verify_trust(&album_root, Some(&expected_id)) {
                        Ok(TrustState::Valid) => return (album_root, mtime_sum, false),
                        _ => return (album_root, mtime_sum, true),
                    }
                }

                match verify_trust(&album_root, Some(&expected_id)) {
                    Ok(TrustState::Valid) => (album_root, mtime_sum, false),
                    Ok(_) => (album_root, mtime_sum, true),
                    Err(e) => {
                        match e {
                            VellumError::ManifestIoError(_) | VellumError::JsonError(_) => {
                                log::debug!("Cache Read Error for {}: {}. Forcing rebuild.", album_root.display(), e);
                            }
                            _ => {}
                        }
                        (album_root, mtime_sum, true)
                    }
                }
            })
            .collect()
    }))
}
