use crate::metrics::{calculate_chroma, calculate_entropy};
use crate::models::{ActionPayload, CoverFileInfo, CoverMetricsDoc, FileStat};
use anyhow::Result;
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use libactions::paths::expand_path;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tempfile::NamedTempFile;

const COVER_CANDIDATES: &[&str] = &[
    "cover.jpg",
    "cover.png",
    "folder.jpg",
    "front.jpg",
    "cover.jpeg",
    "front.png",
];

struct VerifiedCover {
    is_clean: bool,
}

pub fn execute(payload: &ActionPayload) -> Result<()> {
    let music_dir_str = &payload.config.dale.storage.music_directory;
    if music_dir_str.is_empty() {
        anyhow::bail!("music_directory not defined in configuration");
    }

    let music_dir = expand_path(music_dir_str)
        .canonicalize()
        .unwrap_or_else(|_| expand_path(music_dir_str));

    let cache_dir_str = &payload.config.dale.storage.cache;
    let cache_root = if cache_dir_str.is_empty() {
        expand_path("~/.cache/dale")
    } else {
        expand_path(cache_dir_str)
    };

    let lib_hash = blake3::hash(music_dir.to_string_lossy().as_bytes()).to_hex().to_string();
    let library_cache_path = cache_root.join("libraries").join(&lib_hash).join("library.json");
    let library_cache: HashMap<String, FileStat> = fs::read_to_string(&library_cache_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default();

    payload.albums.par_iter().for_each(|album_val| {
        let album_id = album_val
            .pointer("/album/id")
            .or_else(|| album_val.get("id"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        if album_id.is_empty() {
            return;
        }

        let album_dir = music_dir.join(album_id);
        if !album_dir.exists() {
            return;
        }

        let info_dir = album_dir.join("Info");
        let metrics_file = info_dir.join("cover_metrics.json");

        if let Some(target_cover) = verify_existing_metrics(&metrics_file, &info_dir, &music_dir, &library_cache)
            && target_cover.is_clean
        {
            return;
        }

        let Some(cover_path) = resolve_cover_candidate(&album_dir) else {
            return;
        };

        let Ok(cover_meta) = fs::metadata(&cover_path) else {
            return;
        };

        let current_mtime = cover_meta
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let current_size = cover_meta.len();

        let Ok(bytes) = fs::read(&cover_path) else {
            return;
        };

        let raw_hash = blake3::hash(&bytes);
        let hash_str = format!("blake3-{}", STANDARD.encode(raw_hash.as_bytes()));

        let Ok(img) = image::load_from_memory(&bytes) else {
            return;
        };

        let chroma = calculate_chroma(&img);
        let entropy = calculate_entropy(&img);

        let rel_path_for_doc = relative_path_from(&cover_path, &info_dir);

        let doc = CoverMetricsDoc {
            cover: CoverFileInfo {
                path: rel_path_for_doc,
                mtime: current_mtime,
                byte_size: current_size,
                hash: hash_str,
            },
            chroma,
            entropy,
        };

        let Ok(json_str) = serde_json::to_string_pretty(&doc) else {
            return;
        };

        if fs::create_dir_all(&info_dir).is_err() {
            return;
        }

        if let Ok(mut temp) = NamedTempFile::new_in(&info_dir)
            && temp.write_all(json_str.as_bytes()).is_ok()
            && temp.write_all(b"\n").is_ok()
        {
            let _ = temp.persist(&metrics_file);
            println!("\x1b[32m✔\x1b[0m Calculated cover metrics for: {album_id}");
        }
    });

    Ok(())
}

fn verify_existing_metrics(
    metrics_file: &Path,
    info_dir: &Path,
    music_dir: &Path,
    library_cache: &HashMap<String, FileStat>,
) -> Option<VerifiedCover> {
    if !metrics_file.exists() {
        return None;
    }

    let content = fs::read_to_string(metrics_file).ok()?;
    let doc: CoverMetricsDoc = serde_json::from_str(&content).ok()?;

    let resolved_cover = info_dir.join(&doc.cover.path);
    let Ok(meta) = fs::metadata(&resolved_cover) else {
        return Some(VerifiedCover { is_clean: false });
    };

    let mtime = meta
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let size = meta.len();

    if doc.cover.mtime != mtime || doc.cover.byte_size != size {
        return Some(VerifiedCover { is_clean: false });
    }

    if !library_cache.is_empty()
        && let Ok(canon_cover) = resolved_cover.canonicalize()
        && let Ok(rel_lib) = canon_cover.strip_prefix(music_dir)
    {
        let key = rel_lib.to_string_lossy().to_string();
        if let Some(stat) = library_cache.get(&key)
            && (stat.mtime != mtime || stat.size != size)
        {
            return Some(VerifiedCover { is_clean: false });
        }
    }

    Some(VerifiedCover { is_clean: true })
}

fn relative_path_from(target: &Path, base_dir: &Path) -> String {
    if let Ok(rel) = target.strip_prefix(base_dir) {
        return rel.to_string_lossy().to_string();
    }
    if let Some(parent) = base_dir.parent()
        && let Ok(rel) = target.strip_prefix(parent)
    {
        return format!("../{}", rel.to_string_lossy());
    }
    target.to_string_lossy().to_string()
}

fn resolve_cover_candidate(album_dir: &Path) -> Option<PathBuf> {
    for candidate in COVER_CANDIDATES {
        let p = album_dir.join(candidate);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
