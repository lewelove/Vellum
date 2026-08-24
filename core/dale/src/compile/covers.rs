use crate::compile::assets;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CoverCacheEntry {
    pub file_info: Value,
}

pub fn resolve_cover_data(
    album_root: &Path,
    config: &libdale::lua::ResolvedConfig,
) -> Value {
    let main_cover_path = assets::resolve_cover_info(album_root);

    let mut cover_hash_address = String::new();
    let mut cover_file_info = Value::Null;

    if let Some(cp) = &main_cover_path {
        let content = std::fs::read(cp).unwrap_or_default();
        if !content.is_empty() {
            cover_hash_address = libdale::utils::calculate_blake3_address(&content);
            let raw = blake3::hash(&content);
            let cover_hash_full = format!("blake3-{}", STANDARD.encode(raw.as_bytes()));
            let rel_path = libdale::resolvers::rel_path(cp, album_root);
            if let Ok(info) = libdale::utils::get_file_info(cp, &rel_path, false) {
                let mut info_map = info.as_object().unwrap().clone();
                info_map.insert("address".to_string(), json!(cover_hash_address));
                info_map.insert("hash".to_string(), json!(cover_hash_full));
                cover_file_info = Value::Object(info_map);
            }
        }
    }

    assets::pregenerate_covers(config, main_cover_path.as_deref(), &cover_hash_address);

    cover_file_info
}

pub fn resolve_cover_data_cached(
    album_root: &Path,
    config: &libdale::lua::ResolvedConfig,
) -> Value {
    let main_cover_path = assets::resolve_cover_info(album_root);
    let Some(cp) = &main_cover_path else {
        return Value::Null;
    };

    let Ok(meta) = std::fs::metadata(cp) else {
        return Value::Null;
    };

    let mtime = meta
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let size = meta.len();
    let canon_path = cp.canonicalize().unwrap_or_else(|_| cp.clone());

    let cache_key_str = format!("{}:{size}:{mtime}", canon_path.display());
    let key = blake3::hash(cache_key_str.as_bytes()).to_hex().to_string();

    let cache_root = libdale::utils::expand_path(&config.app.storage.cache);
    let cache_dir = cache_root.join("covers").join("metadata");
    let cache_file = cache_dir.join(format!("{key}.json"));

    if let Ok(content) = std::fs::read_to_string(&cache_file) {
        if let Ok(cached) = serde_json::from_str::<CoverCacheEntry>(&content) {
            let cover_hash_address = cached
                .file_info
                .get("address")
                .and_then(Value::as_str)
                .unwrap_or("");
            assets::pregenerate_covers(
                config,
                main_cover_path.as_deref(),
                cover_hash_address,
            );
            return cached.file_info;
        }
        let _ = std::fs::remove_file(&cache_file);
    }

    let file_info = resolve_cover_data(album_root, config);

    let entry = CoverCacheEntry {
        file_info: file_info.clone(),
    };
    if let Ok(json_str) = serde_json::to_string(&entry) {
        let _ = libdale::utils::write_atomic_cache_file(&cache_file, &json_str);
    }

    file_info
}
