use crate::metrics::{calculate_chroma, calculate_entropy};
use crate::models::{CoverFileInfo, CoverMetricsDoc};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use libactions::payload::ActionPayload;
use rayon::prelude::*;
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

pub fn execute(payload: &ActionPayload) {
    payload.albums.par_iter().for_each(|item| {
        let album_dir = &item.path;
        let album_val = &item.lock;
        let album_id = album_val
            .pointer("/album/id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        if !album_dir.exists() {
            return;
        }

        let info_dir = album_dir.join("Info");
        let metrics_file = info_dir.join("cover_metrics.json");

        if let Some(target_cover) = verify_existing_metrics(&metrics_file, &info_dir)
            && target_cover.is_clean
        {
            return;
        }

        let cover_rel_path = album_val
            .pointer("/album/covers/main/file/path")
            .and_then(serde_json::Value::as_str);

        let cover_path = cover_rel_path
            .map(|rel| album_dir.join(rel))
            .filter(|p| p.is_file())
            .or_else(|| resolve_cover_candidate(album_dir));

        let Some(cover_path) = cover_path else {
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
}

fn verify_existing_metrics(
    metrics_file: &Path,
    info_dir: &Path,
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
