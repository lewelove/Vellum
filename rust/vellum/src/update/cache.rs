use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlbumCacheEntry {
    pub mtime_sum: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentState {
    pub hash: String,
}

pub fn calculate_hash(data: &str) -> String {
    blake3::hash(data.as_bytes()).to_hex().to_string()
}

pub fn get_lua_config_hash(dependencies: &[PathBuf]) -> String {
    let mut hasher = blake3::Hasher::new();
    for dep in dependencies {
        if let Ok(content) = fs::read(dep) {
            hasher.update(&content);
        }
    }
    hasher.finalize().to_hex().to_string()
}

pub fn load_cache(path: &Path) -> HashMap<String, AlbumCacheEntry> {
    if let Ok(content) = fs::read_to_string(path)
        && let Ok(cache) = serde_json::from_str::<HashMap<String, AlbumCacheEntry>>(&content)
    {
        return cache;
    }
    HashMap::new()
}

pub fn save_cache(cache: &HashMap<String, AlbumCacheEntry>, path: &Path) -> Result<()> {
    let content = serde_json::to_string_pretty(cache)?;
    fs::write(path, content)?;
    Ok(())
}

pub async fn validate_library_root(cache_dir: &Path, current_hash: &str) -> Result<()> {
    let current_json_path = cache_dir.join("current.json");
    if current_json_path.exists() {
        let content = fs::read_to_string(&current_json_path).unwrap_or_default();
        let saved_state: Result<CurrentState, _> = serde_json::from_str(&content);
        if let Ok(state) = saved_state && state.hash == current_hash {
            return Ok(());
        }
    }

    let state = CurrentState {
        hash: current_hash.to_string(),
    };
    let content = serde_json::to_string(&state)?;
    let _ = fs::write(&current_json_path, content);
    let _ = trigger_server_reset().await;
    Ok(())
}

pub fn get_mtime_sum(dir: &Path, meta: &Path, exts: &[String], manifests: Option<&Vec<String>>) -> u64 {
    let d_mtime = fs::metadata(dir)
        .and_then(|m| m.modified())
        .map_or(0, |t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

    let mut m_mtime = fs::metadata(meta)
        .and_then(|m| m.modified())
        .map_or(0, |t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

    if let Some(names) = manifests {
        for name in names {
            let p = dir.join(name);
            if p.exists() {
                m_mtime += fs::metadata(&p)
                    .and_then(|m| m.modified())
                    .map_or(0, |t| {
                        t.duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    });
            }
        }
    }

    let mut c_mtime = 0;
    let cover_candidates = ["cover.jpg", "cover.png", "folder.jpg", "front.jpg"];

    for c in cover_candidates {
        let cp = dir.join(c);
        if cp.exists() {
            c_mtime = fs::metadata(cp)
                .and_then(|m| m.modified())
                .map_or(0, |t| {
                    t.duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                });
            break;
        }
    }

    let mut t_mtime = 0;
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(3)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.is_file()
            && let Some(ext) = p.extension().and_then(|e| e.to_str())
        {
            let ext_lower = format!(".{}", ext.to_lowercase());
            if exts.contains(&ext_lower) {
                t_mtime += entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map_or(0, |t| {
                        t.duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    });
            }
        }
    }

    d_mtime + m_mtime + c_mtime + t_mtime
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
