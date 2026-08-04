use crate::compile::assets;
use libvellum::models::CoverMetrics;
use serde_json::{Value, json};
use std::path::Path;
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CoverCacheEntry {
    pub file_info: Value,
    pub metrics: Option<CoverMetrics>,
}

pub fn resolve_cover_data(
    album_root: &Path,
    config: &libvellum::lua::ResolvedConfig,
    lib_hash: &str,
) -> (Value, Option<CoverMetrics>) {
    let main_cover_path = assets::resolve_cover_info(album_root);
    
    let mut cover_hash_address = String::new();
    let mut cover_file_info = Value::Null;

    if let Some(cp) = &main_cover_path {
        let content = std::fs::read(cp).unwrap_or_default();
        if !content.is_empty() {
            cover_hash_address = libvellum::utils::calculate_blake3_address(&content);
            let raw = blake3::hash(&content);
            let cover_hash_full = format!("blake3-{}", STANDARD.encode(raw.as_bytes()));
            let rel_path = libvellum::resolvers::rel_path(cp, album_root);
            if let Ok(info) = libvellum::utils::get_file_info(cp, &rel_path, false) {
                let mut info_map = info.as_object().unwrap().clone();
                info_map.insert("address".to_string(), json!(cover_hash_address));
                info_map.insert("hash".to_string(), json!(cover_hash_full));
                cover_file_info = Value::Object(info_map);
            }
        }
    }

    let loaded_image = assets::pregenerate_covers(config, main_cover_path.as_deref(), &cover_hash_address);
    let cover_metrics = resolve_cover_metrics(config, lib_hash, &cover_hash_address, loaded_image.as_ref());

    (cover_file_info, cover_metrics)
}

pub fn resolve_cover_data_cached(
    album_root: &Path,
    config: &libvellum::lua::ResolvedConfig,
    lib_hash: &str,
) -> (Value, Option<CoverMetrics>) {
    let main_cover_path = assets::resolve_cover_info(album_root);
    let Some(cp) = &main_cover_path else {
        return (Value::Null, None);
    };

    let Ok(meta) = std::fs::metadata(cp) else {
        return (Value::Null, None);
    };

    let mtime = meta
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let size = meta.len();
    let canon_path = cp.canonicalize().unwrap_or_else(|_| cp.clone());

    let cache_key_str = format!("{}:{size}:{mtime}", canon_path.display());
    let key = blake3::hash(cache_key_str.as_bytes()).to_hex().to_string();

    let cache_root = libvellum::utils::expand_path(&config.app.storage.cache);
    let cache_dir = cache_root.join("covers").join("metadata");
    let cache_file = cache_dir.join(format!("{key}.json"));

    if cache_file.exists()
        && let Ok(content) = std::fs::read_to_string(&cache_file)
        && let Ok(cached) = serde_json::from_str::<CoverCacheEntry>(&content)
    {
        return (cached.file_info, cached.metrics);
    }

    let (file_info, metrics) = resolve_cover_data(album_root, config, lib_hash);

    if std::fs::create_dir_all(&cache_dir).is_ok() {
        let entry = CoverCacheEntry {
            file_info: file_info.clone(),
            metrics: metrics.clone(),
        };
        if let Ok(json_str) = serde_json::to_string(&entry) {
            let _ = std::fs::write(cache_file, json_str);
        }
    }

    (file_info, metrics)
}

pub fn resolve_cover_metrics(
    config: &libvellum::lua::ResolvedConfig,
    lib_hash: &str,
    c_hash: &str,
    loaded_image: Option<&image::DynamicImage>,
) -> Option<CoverMetrics> {
    if c_hash.is_empty() {
        return None;
    }
    
    let cache_root = libvellum::utils::expand_path(&config.app.storage.cache);
    let metrics_dir = cache_root.join("libraries").join(lib_hash).join("covers_data");
    std::fs::create_dir_all(&metrics_dir).ok();
    
    let metrics_path = metrics_dir.join(format!("{c_hash}.json"));
    
    let mut metrics = if metrics_path.exists() {
        std::fs::read_to_string(&metrics_path).map_or(None, |content| serde_json::from_str::<CoverMetrics>(&content).ok())
    } else { 
        None 
    }.unwrap_or_else(|| CoverMetrics {
        hash: c_hash.to_string(),
        entropy: None,
        chroma: None,
    });
    
    let mut needs_save = false;
    
    if let Some(img) = loaded_image {
        if metrics.chroma.is_none() {
            metrics.chroma = Some(libvellum::images::cover_chroma::calculate_chroma(img));
            needs_save = true;
        }
        if metrics.entropy.is_none() {
            metrics.entropy = Some(libvellum::images::cover_entropy::calculate_entropy(img));
            needs_save = true;
        }
    }
    
    if needs_save
        && let Ok(content) = serde_json::to_string(&metrics) {
            let _ = std::fs::write(&metrics_path, content);
        }
    
    Some(metrics)
}
