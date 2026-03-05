use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::compile;
use crate::config::AppConfig;
use crate::expand_path;

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
    no_extensions: bool,
) -> Result<()> {
    let (config, _, _) = AppConfig::load().context("Failed to load config")?;
    let library_root = expand_path(&config.storage.library_root)
        .canonicalize()
        .context("Invalid library_root")?;

    let lib_hash = calculate_hash(&library_root.to_string_lossy());
    let base_cache_dir = expand_path("~/.vellum/libraries_cache");
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
    let all_albums = compile::builder::scan::find_target_albums(&scan_root, scan_depth);

    log::info!("Verifying {} albums...", all_albums.len());

    let results = verify_albums_parallel(all_albums, &cache, force, jobs)?;

    let mut work_queue = Vec::new();
    let mut trusted_count = 0;

    for (path, mtime, is_dirty) in results {
        if is_dirty {
            work_queue.push(path);
        } else {
            cache.insert(
                path.to_string_lossy().to_string(),
                AlbumCacheEntry { mtime_sum: mtime },
            );
            trusted_count += 1;
        }
    }

    if work_queue.is_empty() {
        log::info!("Library is up to date ({trusted_count} albums trusted).");
        save_cache(&cache, &cache_file)?;
        return Ok(());
    }

    log::info!(
        "Processing {} dirty albums ({} trusted).",
        work_queue.len(),
        trusted_count
    );

    let (notify_tx, mut notify_rx) = mpsc::channel::<PathBuf>(100);
    let cache_arc = Arc::new(Mutex::new(cache));
    let cache_for_task = Arc::clone(&cache_arc);

    let notification_task = tokio::spawn(async move {
        let client = reqwest::Client::new();
        while let Some(album_root) = notify_rx.recv().await {
            handle_album_reload(&client, &album_root, &cache_for_task).await;
        }
    });

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
            no_extensions,
        },
    };

    compile::run(compile_options).await?;

    drop(notify_tx);
    let _ = notification_task.await;

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
                let mtime_sum = get_mtime_sum(&album_root, &metadata_path);

                if force {
                    return (album_root, mtime_sum, true);
                }

                if let Some(entry) = cache.get(&album_path_str)
                    && entry.mtime_sum == mtime_sum
                    && mtime_sum != 0
                {
                    return (album_root, mtime_sum, false);
                }

                match verify_trust(&album_root) {
                    TrustState::Valid => (album_root, mtime_sum, false),
                    _ => (album_root, mtime_sum, true),
                }
            })
            .collect()
    }))
}

async fn handle_album_reload(
    client: &reqwest::Client,
    album_root: &Path,
    cache: &Arc<Mutex<HashMap<String, AlbumCacheEntry>>>,
) {
    let album_path_str = album_root.to_string_lossy().to_string();
    let metadata_path = album_root.join("metadata.toml");
    let mtime_sum = get_mtime_sum(album_root, &metadata_path);

    let params = [("path", album_path_str.clone())];
    let _ = client
        .post("http://127.0.0.1:8000/api/internal/reload")
        .query(&params)
        .timeout(std::time::Duration::from_millis(1000))
        .send()
        .await;

    let mut c = cache.lock().await;
    c.insert(album_path_str, AlbumCacheEntry { mtime_sum });
}

fn calculate_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_mtime_sum(dir: &Path, meta: &Path) -> u64 {
    let d_mtime = fs::metadata(dir)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0);

    let m_mtime = fs::metadata(meta)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0);

    let mut c_mtime = 0;
    let cover_candidates = ["cover.jpg", "cover.png", "folder.jpg", "front.jpg"];

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

    d_mtime + m_mtime + c_mtime
}

fn verify_trust(album_root: &Path) -> TrustState {
    let lock_path = album_root.join("metadata.lock.json");
    if !lock_path.exists() {
        return TrustState::Missing;
    }

    let Ok(lock_content) = fs::read_to_string(&lock_path) else {
        return TrustState::Missing;
    };

    let lock_json: serde_json::Value = match serde_json::from_str(&lock_content) {
        Ok(j) => j,
        Err(_) => return TrustState::Missing,
    };

    let Some(album_data) = lock_json.get("album") else {
        return TrustState::Missing;
    };

    let lock_mtime = album_data
        .get("metadata_toml_mtime")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let metadata_path = album_root.join("metadata.toml");
    let current_mtime = fs::metadata(&metadata_path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0);

    if current_mtime != lock_mtime && lock_mtime != 0 {
        return TrustState::BrokenIntent;
    }

    let lock_cover_path = album_data
        .get("cover_path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if !lock_cover_path.is_empty() && lock_cover_path != "default_cover.png" {
        let abs_cover = album_root.join(lock_cover_path);
        if !abs_cover.exists() {
            return TrustState::BrokenAssets;
        }

        let lock_cover_size = album_data
            .get("cover_byte_size")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let current_cover_size = fs::metadata(&abs_cover).map(|m| m.len()).unwrap_or(0);

        if lock_cover_size != current_cover_size {
            return TrustState::BrokenAssets;
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
                return TrustState::BrokenPhysics;
            }

            let abs_path = album_root.join(rel_path);
            let Ok(meta) = fs::metadata(&abs_path) else {
                return TrustState::BrokenPhysics;
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
                return TrustState::BrokenPhysics;
            }
        }
    }

    TrustState::Valid
}

fn load_cache(path: &Path) -> HashMap<String, AlbumCacheEntry> {
    if let Ok(content) = fs::read_to_string(path)
        && let Ok(cache) = serde_json::from_str::<HashMap<String, AlbumCacheEntry>>(&content)
    {
        return cache;
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
