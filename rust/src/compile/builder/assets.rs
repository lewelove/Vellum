use crate::expand_path;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use fast_image_resize::PixelType;
use image::DynamicImage;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path};
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

pub fn generate_master_blob(original_path: &Path, master_blob_path: &Path) -> anyhow::Result<()> {
    let img = image::open(original_path)?.into_rgb8();
    let src_width = img.width();
    let src_height = img.height();

    let min_dim = std::cmp::min(src_width, src_height);
    let src_image = Image::from_vec_u8(
        src_width,
        src_height,
        img.into_raw(),
        PixelType::U8x3,
    ).map_err(|e| anyhow::anyhow!("Resize src init failed: {}", e))?;

    let mut dst_image = Image::new(
        1080,
        1080,
        PixelType::U8x3,
    );

    let mut resizer = Resizer::new();
    let options = ResizeOptions::new()
        .crop(
            ((src_width - min_dim) / 2) as f64,
            ((src_height - min_dim) / 2) as f64,
            min_dim as f64,
            min_dim as f64,
        )
        .resize_alg(ResizeAlg::Convolution(FilterType::Mitchell));
    
    resizer.resize(&src_image, &mut dst_image, &options)
        .map_err(|e| anyhow::anyhow!("Resize execution failed: {}", e))?;

    if let Some(parent) = master_blob_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(master_blob_path, dst_image.into_vec())?;
    
    Ok(())
}

pub fn load_or_create_thumbnail(
    config: &Value,
    album_root: &Path,
    cover_path: Option<&str>,
    cover_hash: &str,
) -> Option<DynamicImage> {
    let storage = config.get("storage")?;
    let cache_str = storage
        .get("cache")
        .and_then(Value::as_str)?;
    let cp = cover_path?;
    if cover_hash.is_empty() {
        return None;
    }

    let size = config
        .get("theme")
        .and_then(|t| t.get("thumbnail_size"))
        .and_then(Value::as_u64)
        .map_or(200, |s| u32::try_from(s).unwrap_or(200)) as u32;

    let cache_root = expand_path(cache_str);
    let master_blob_path = cache_root.join("covers").join(format!("{cover_hash}.rgb"));

    if !master_blob_path.exists() {
        let original_path = album_root.join(cp);
        let _ = generate_master_blob(&original_path, &master_blob_path);
    }

    let thumb_dir = cache_root.join("thumbnails").join(format!("{size}px"));
    let thumb_path = thumb_dir.join(format!("{cover_hash}.png"));

    if !thumb_path.exists() {
        if let Ok(master_bytes) = std::fs::read(&master_blob_path) {
            let src_image = Image::from_vec_u8(
                1080,
                1080,
                master_bytes,
                PixelType::U8x3,
            ).ok()?;

            let mut dst_image = Image::new(
                size,
                size,
                PixelType::U8x3,
            );

            let mut resizer = Resizer::new();
            let options = ResizeOptions::new()
                .resize_alg(ResizeAlg::Convolution(FilterType::Mitchell));

            resizer.resize(&src_image, &mut dst_image, &options).ok()?;

            let result_buf = dst_image.into_vec();
            let img_buffer = image::RgbImage::from_raw(size, size, result_buf)?;
            let final_thumb = DynamicImage::ImageRgb8(img_buffer);
            
            let _ = std::fs::create_dir_all(&thumb_dir);
            let _ = final_thumb.save(&thumb_path);
            return Some(final_thumb);
        }
    } else if let Ok(img) = image::open(&thumb_path) {
        return Some(img);
    }
    None
}
