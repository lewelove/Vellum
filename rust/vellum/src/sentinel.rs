use crate::error::VellumError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
pub enum TrustState {
    Valid,
    Missing,
    BrokenIntent,
    BrokenPhysics,
    BrokenAssets,
}

pub fn verify_trust(album_root: &Path, manifests: &Option<Vec<String>>) -> Result<TrustState, VellumError> {
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
