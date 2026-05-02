use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::compile;
use vellum_core::config::AppConfig;
use vellum_core::error::VellumError;
use vellum_core::utils::expand_path;

#[derive(Serialize, Deserialize, Debug)]
pub enum TrustState {
    Valid,
    Missing,
    BrokenIntent,
    BrokenPhysics,
    BrokenAssets,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AlbumCacheEntry {
    pub mtime_sum: u64,
}

#[derive(Serialize, Deserialize)]
struct CurrentState {
    pub hash: String,
}

pub async fn run(
    target_path: Option<PathBuf>,
    force: bool,
    jobs: Option<usize>,
    verbose: bool,
    silent: bool,
) -> Result<()> {
    let (config, _, _): (AppConfig, toml::Value, PathBuf) = AppConfig::load().context("Failed to load config")?;
    let library_root = expand_path(&config.storage.library_root)
        .canonicalize()
        .context("Invalid library_root")?;

    let is_full_library = target_path.is_none() || target_path.as_deref() == Some(library_root.as_path());

    if force && is_full_library {
        let client = reqwest::Client::new();
        let _ = client
            .post("http://127.0.0.1:8000/api/internal/notify_force_update")
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;
    }

    let exts = config
        .manifest
        .as_ref()
        .and_then(|m| m.supported_extensions.clone())
        .unwrap_or_else(|| vec![".flac".to_string()]);

    let manifests = config.compiler.as_ref().and_then(|c| c.manifests.clone());

    let lib_hash = calculate_hash(&library_root.to_string_lossy());
    let base_cache_dir = expand_path(&config.storage.cache).join("libraries_cache");
    fs::create_dir_all(&base_cache_dir)?;

    validate_library_root(&base_cache_dir, &lib_hash).await?;

    let cache_file = base_cache_dir.join(format!("{lib_hash}.json"));
    let mut cache = load_cache(&cache_file);

    let scan_root = target_path.unwrap_or_else(|| library_root.clone());
    let scan_depth = config
        .compiler
        .as_ref()
        .and_then(|c| c.scan_depth)
        .unwrap_or(4);
    
    let all_albums = compile::builder::scan::find_target_albums(&scan_root, scan_depth)?;
    
    let mut missing_paths = Vec::new();
    {
        let album_set: HashSet<PathBuf> = all_albums.iter().cloned().collect();
        let scan_root_canon = scan_root.canonicalize().unwrap_or(scan_root.clone());
        
        for cached_path_str in cache.keys() {
            let cached_path = PathBuf::from(cached_path_str);
            if cached_path.starts_with(&scan_root_canon) && !album_set.contains(&cached_path) {
                missing_paths.push(cached_path);
            }
        }
    }

    if !silent {
        log::info!("Verifying {} albums...", all_albums.len());
    }

    let results = verify_albums_parallel(all_albums, &cache, force, jobs, &exts, &manifests)?;

    let mut work_queue = Vec::new();

    for (path, mtime, is_dirty) in results {
        if is_dirty {
            work_queue.push(path);
        } else {
            cache.insert(
                path.to_string_lossy().to_string(),
                AlbumCacheEntry { mtime_sum: mtime },
            );
        }
    }

    if work_queue.is_empty() && missing_paths.is_empty() {
        if !silent {
            log::info!("Library is up to date.");
        }
        save_cache(&cache, &cache_file)?;
        return Ok(());
    }

    let dirty_count = work_queue.len();
    let missing_count = missing_paths.len();
    let start_time = std::time::Instant::now();

    let (notify_tx, mut notify_rx) = mpsc::channel::<compile::engine::stream::AlbumUpdateSignal>(100);
    let cache_arc = Arc::new(Mutex::new(cache));
    let cache_for_task = Arc::clone(&cache_arc);
    let exts_for_task = exts.clone();
    let manifests_for_task = manifests.clone();
    let lib_root_for_task = Arc::new(library_root.clone());

    let notification_task = tokio::spawn(async move {
        let mut updated_paths = Vec::new();
        while let Some(signal) = notify_rx.recv().await {
            if verbose && !silent {
                log::info!("Updated: {} - {}", signal.artist, signal.album);
            }
            updated_paths.push(signal.path);
        }

        let mut paths_for_server = Vec::new();

        let mut c = cache_for_task.lock().await;
        for album_root in &updated_paths {
            let album_path_str = album_root.to_string_lossy().to_string();
            let metadata_path = album_root.join("metadata.toml");
            let mtime_sum = get_mtime_sum(album_root, &metadata_path, &exts_for_task, &manifests_for_task);
            c.insert(album_path_str.clone(), AlbumCacheEntry { mtime_sum });
            paths_for_server.push(album_path_str);
        }

        for missing in missing_paths {
            let p_str = missing.to_string_lossy().to_string();
            
            if verbose && !silent {
                let display_path = missing.strip_prefix(&*lib_root_for_task).unwrap_or(&missing);
                log::info!("Removed: {}", display_path.display());
            }

            c.remove(&p_str);
            paths_for_server.push(p_str);
        }

        if paths_for_server.is_empty() {
            return;
        }

        let client = reqwest::Client::new();
        let elapsed_ms = start_time.elapsed().as_millis();
        let url = format!("http://127.0.0.1:8000/api/internal/batch_reload?time={}", elapsed_ms);

        let _ = client
            .post(&url)
            .json(&paths_for_server)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;
    });

    if !work_queue.is_empty() {
        let compile_options = compile::CompileOptions {
            target_path: scan_root,
            flags: vec!["default".to_string()],
            specific_albums: Some(work_queue),
            jobs,
            notify_tx: Some(notify_tx.clone()),
            compile_flags: compile::CompileFlags {
                mode: compile::CompileMode::Standard,
                target: compile::ExportTarget::File,
                pretty: false,
            },
        };
        compile::run(compile_options).await?;
    }

    drop(notify_tx);
    let _ = notification_task.await;

    let elapsed = start_time.elapsed().as_millis();
    if !silent {
        log::info!(
            "Update complete: {} updated, {} removed. Finished in {}ms.",
            dirty_count,
            missing_count,
            elapsed
        );
    }

    let final_cache = Arc::try_unwrap(cache_arc).unwrap().into_inner();
    save_cache(&final_cache, &cache_file)?;

    Ok(())
}

async fn validate_library_root(cache_dir: &Path, current_hash: &str) -> Result<()> {
    let current_json_path = cache_dir.join("current.json");
    let mut needs_reset = false;

    if current_json_path.exists() {
        let content = fs::read_to_string(&current_json_path).unwrap_or_default();
        if let Ok(state) = serde_json::from_str::<CurrentState>(&content) {
            if state.hash != current_hash {
                needs_reset = true;
            }
        } else {
            needs_reset = true;
        }
    } else {
        needs_reset = true;
    }

    if needs_reset {
        log::info!("Library root changed. Triggering server reset.");
        let _ = fs::write(
            &current_json_path,
            serde_json::to_string(&CurrentState {
                hash: current_hash.to_string(),
            })?,
        );
        let _ = trigger_server_reset().await;
    }
    Ok(())
}

fn verify_albums_parallel(
    albums: Vec<PathBuf>,
    cache: &HashMap<String, AlbumCacheEntry>,
    force: bool,
    jobs: Option<usize>,
    exts: &[String],
    manifests: &Option<Vec<String>>,
) -> Result<Vec<(PathBuf, u64, bool)>> {
    let default_parallelism = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1);
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

                if let Some(entry) = cache.get(&album_path_str)
                {
                    if entry.mtime_sum == mtime_sum && mtime_sum != 0 {
                         return (album_root, mtime_sum, false);
                    }
                }

                match verify_trust(&album_root, manifests) {
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

fn calculate_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_mtime_sum(dir: &Path, meta: &Path, exts: &[String], manifests: &Option<Vec<String>>) -> u64 {
    let d_mtime = fs::metadata(dir)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0);

    let mut m_mtime = fs::metadata(meta)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0);

    if let Some(names) = manifests {
        for name in names {
            let p = dir.join(name);
            if p.exists() {
                m_mtime += fs::metadata(&p)
                    .and_then(|m| m.modified())
                    .map(|t| {
                        t.duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    })
                    .unwrap_or(0);
            }
        }
    }

    let mut c_mtime = 0;
    let cover_candidates =["cover.jpg", "cover.png", "folder.jpg", "front.jpg"];

    for c in cover_candidates {
        let cp = dir.join(c);
        if cp.exists() {
            c_mtime = fs::metadata(cp)
                .and_then(|m| m.modified())
                .map(|t| {
                    t.duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0);
            break;
        }
    }

    let mut t_mtime = 0;
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                let ext_lower = format!(".{}", ext.to_lowercase());
                if exts.contains(&ext_lower) {
                    t_mtime += entry
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            t.duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                        })
                        .unwrap_or(0);
                }
            }
        }
    }

    d_mtime + m_mtime + c_mtime + t_mtime
}

fn verify_trust(album_root: &Path, manifests: &Option<Vec<String>>) -> Result<TrustState, VellumError> {
    let lock_path = album_root.join("metadata.lock.json");
    if !lock_path.exists() {
        return Ok(TrustState::Missing);
    }

    let lock_content = fs::read_to_string(&lock_path)
        .map_err(VellumError::ManifestIoError)?;

    let lock_json: serde_json::Value = serde_json::from_str(&lock_content)
        .map_err(VellumError::JsonError)?;

    let Some(album_data) = lock_json.get("album") else {
        return Ok(TrustState::Missing);
    };

    let lock_meta_mtime = album_data
        .get("metadata_toml_mtime")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    let lock_manifests_sum = album_data
        .get("manifests_mtime_sum")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    
    let metadata_path = album_root.join("metadata.toml");
    let current_meta_mtime = fs::metadata(&metadata_path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0);

    if current_meta_mtime != lock_meta_mtime && lock_meta_mtime != 0 {
        return Ok(TrustState::BrokenIntent);
    }

    let mut current_manifests_sum: u64 = current_meta_mtime;
    if let Some(names) = manifests {
        for name in names {
            let p = album_root.join(name);
            if p.exists() {
                current_manifests_sum += fs::metadata(&p)
                    .and_then(|m| m.modified())
                    .map(|t| {
                        t.duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    })
                    .unwrap_or(0);
            }
        }
    }

    if current_manifests_sum != lock_manifests_sum && lock_manifests_sum != 0 {
        return Ok(TrustState::BrokenIntent);
    }

    let lock_cover_path = album_data
        .get("cover_path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if !lock_cover_path.is_empty() && lock_cover_path != "default_cover.png" {
        let abs_cover = album_root.join(lock_cover_path);
        if !abs_cover.exists() {
            return Ok(TrustState::BrokenAssets);
        }

        let lock_cover_size = album_data
            .get("cover_byte_size")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let current_cover_size = fs::metadata(&abs_cover).map(|m| m.len()).unwrap_or(0);

        if lock_cover_size != current_cover_size {
            return Ok(TrustState::BrokenAssets);
        }
    }

    if let Some(tracks) = lock_json
        .get("tracks")
        .and_then(serde_json::Value::as_array)
    {
        for track in tracks {
            let rel_path = track
                .get("track_path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if rel_path.is_empty() {
                return Ok(TrustState::BrokenPhysics);
            }

            let abs_path = album_root.join(rel_path);
            let Ok(meta) = fs::metadata(&abs_path) else {
                return Ok(TrustState::BrokenPhysics);
            };

            let lock_track_mtime = track
                .get("track_mtime")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            let lock_track_size = track
                .get("track_size")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);

            let current_track_mtime = meta
                .modified()
                .map(|t| {
                    t.duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0);
            let current_track_size = meta.len();

            if lock_track_mtime != current_track_mtime || lock_track_size != current_track_size {
                return Ok(TrustState::BrokenPhysics);
            }
        }
    }

    Ok(TrustState::Valid)
}

fn load_cache(path: &Path) -> HashMap<String, AlbumCacheEntry> {
    if let Ok(content) = fs::read_to_string(path)
    {
        if let Ok(cache) = serde_json::from_str::<HashMap<String, AlbumCacheEntry>>(&content) {
             return cache;
        }
    }
    HashMap::new()
}

fn save_cache(cache: &HashMap<String, AlbumCacheEntry>, path: &Path) -> Result<()> {
    let content = serde_json::to_string_pretty(cache)?;
    fs::write(path, content)?;
    Ok(())
}

async fn trigger_server_reset() -> Result<()> {
    let client = reqwest::Client::new();
    let _ = client
        .post("http://127.0.0.1:8000/api/internal/reset")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await;
    Ok(())
}
