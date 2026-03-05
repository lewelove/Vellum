use crate::expand_path;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use image::{DynamicImage, GenericImageView};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::SystemTime;

pub fn resolve_cover_info(root: &Path) -> (Option<String>, String, u64, u64) {
    let candidates = ["cover.jpg", "cover.png", "folder.jpg", "front.jpg"];
    for c in candidates {
        let p = root.join(c);
        if let Ok(m) = std::fs::metadata(&p) {
            let mtime = m
                .modified()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let size = m.len();
            let mut h = Sha256::new();
            h.update(mtime.to_be_bytes());
            h.update(size.to_be_bytes());
            return (
                Some(c.to_string()),
                URL_SAFE_NO_PAD.encode(&h.finalize()[..8]),
                mtime,
                size,
            );
        }
    }
    (None, String::new(), 0, 0)
}

pub fn load_or_create_thumbnail(
    config: &Value,
    album_root: &Path,
    cover_path: Option<&str>,
    cover_hash: &str,
) -> Option<DynamicImage> {
    let storage = config.get("storage")?;
    let dir_str = storage
        .get("thumbnail_cache_folder")
        .and_then(Value::as_str)?;
    let cp = cover_path?;
    if cover_hash.is_empty() {
        return None;
    }

    let size = config
        .get("theme")
        .and_then(|t| t.get("thumbnail_size"))
        .and_then(Value::as_u64)
        .map_or(200, |s| u32::try_from(s).unwrap_or(200));

    let thumb_dir = expand_path(dir_str).join(format!("{size}px"));
    let thumb_path = thumb_dir.join(format!("{cover_hash}.png"));

    if !thumb_path.exists() {
        if let Ok(img) = image::open(album_root.join(cp)) {
            let (w, h) = img.dimensions();
            let min_dimension = std::cmp::min(w, h);
            let square = img.crop_imm(
                (w - min_dimension) / 2,
                (h - min_dimension) / 2,
                min_dimension,
                min_dimension,
            );
            let final_thumb = square.resize(size, size, image::imageops::FilterType::Lanczos3);
            let _ = std::fs::create_dir_all(&thumb_dir);
            let _ = final_thumb.save(&thumb_path);
            return Some(final_thumb);
        }
    } else if let Ok(img) = image::open(&thumb_path) {
        return Some(img);
    }
    None
}
