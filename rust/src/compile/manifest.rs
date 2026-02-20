use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use xxhash_rust::xxh64::Xxh64;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use image::{DynamicImage, GenericImageView};

use crate::compile::resolve::{self, AlbumContext, TrackContext};
use crate::compile::scan;
use crate::harvest;
use crate::expand_path;

pub fn build(
    album_root: &Path,
    project_root: &Path,
    config: &Value,
    gen_cfg: &Value,
) -> Result<Value> {
    let metadata_path = album_root.join("metadata.toml");
    
    let (metadata_mtime, metadata_hash) = {
        let meta = std::fs::metadata(&metadata_path)?;
        let mtime = meta.modified()?.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
        
        let file = File::open(&metadata_path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        std::io::copy(&mut reader, &mut hasher)?;
        (mtime, format!("{:x}", hasher.finalize()))
    };

    let content = std::fs::read_to_string(&metadata_path)?;
    let metadata_toml: toml::Value = toml::from_str(&content)?;
    let metadata_json = serde_json::to_value(metadata_toml)?;

    let (cover_path, cover_hash) = resolve_cover_info(album_root);
    
    // Thumbnail Generation Logic
    let storage = config.get("storage").ok_or_else(|| anyhow!("Missing [storage]"))?;
    let thumb_dir_raw = storage.get("thumbnail_cache_folder").and_then(|v| v.as_str());
    
    let mut loaded_image: Option<DynamicImage> = None;

    if let (Some(dir_str), Some(cp), false) = (thumb_dir_raw, &cover_path, cover_hash.is_empty()) {
        let thumb_dir = expand_path(dir_str);
        let thumb_path = thumb_dir.join(format!("{}.png", cover_hash));

        if !thumb_path.exists() {
            if let Ok(img) = image::open(album_root.join(cp)) {
                let (w, h) = img.dimensions();
                let side = std::cmp::min(w, h);
                let mut square = img.crop_imm((w - side) / 2, (h - side) / 2, side, side);
                
                // Resize to standard 200px thumbnail for analysis and storage
                let final_thumb = square.resize(200, 200, image::imageops::FilterType::Lanczos3);
                
                let _ = std::fs::create_dir_all(&thumb_dir);
                let _ = final_thumb.save(&thumb_path);
                loaded_image = Some(final_thumb);
            }
        } else if let Ok(img) = image::open(&thumb_path) {
            loaded_image = Some(img);
        }
    }

    let supported: Vec<String> = gen_cfg.get("supported_extensions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec![".flac".to_string()]);
    
    let exts: Vec<&str> = supported.iter().map(|s| s.as_str()).collect();
    let audio_files = scan::scan_audio_files(album_root, &exts);

    let album_src = metadata_json.get("album").cloned().unwrap_or(json!({}));
    let track_entries = metadata_json.get("tracks").and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing [[tracks]] in metadata.toml for {:?}", album_root))?;

    if audio_files.len() != track_entries.len() {
        return Err(anyhow!("Manifest Mismatch in {:?}: {} files vs {} entries", album_root, audio_files.len(), track_entries.len()));
    }

    let reg_cfg = config.get("compiler_registry").ok_or_else(|| anyhow!("Missing [compiler_registry]"))?;
    let album_reg: HashSet<String> = reg_cfg.get("album").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let track_reg: HashSet<String> = reg_cfg.get("tracks").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let library_root = Path::new(storage.get("library_root").and_then(|v| v.as_str()).unwrap_or("."));

    let mut harvested_spine = Vec::with_capacity(audio_files.len());
    for path in &audio_files {
        harvested_spine.push(harvest::harvest_file(path)?);
    }

    harvested_spine.sort_by(|a, b| {
        let a_disc = parse_tag_int(a.tags.get("DISCNUMBER"));
        let b_disc = parse_tag_int(b.tags.get("DISCNUMBER"));
        if a_disc != b_disc { return a_disc.cmp(&b_disc); }
        let a_track = parse_tag_int(a.tags.get("TRACKNUMBER"));
        let b_track = parse_tag_int(b.tags.get("TRACKNUMBER"));
        if a_track != b_track { return a_track.cmp(&b_track); }
        a.path.cmp(&b.path)
    });

    let mut final_tracks = Vec::with_capacity(audio_files.len());
    let mut harvested_cache = Vec::with_capacity(audio_files.len());
    let mut current_physical_disc = None;
    let mut ordinal_disc_counter = 0;
    let mut ordinal_track_counter = 0;

    for (idx, harvest_data) in harvested_spine.into_iter().enumerate() {
        let phys_disc = parse_tag_int(harvest_data.tags.get("DISCNUMBER"));
        if Some(phys_disc) != current_physical_disc {
            current_physical_disc = Some(phys_disc);
            ordinal_disc_counter += 1;
            ordinal_track_counter = 1;
        } else {
            ordinal_track_counter += 1;
        }

        let entry = &track_entries[idx];
        let mut track_obj = serde_json::Map::new();
        let track_ctx = TrackContext {
            ordinal_track_number: ordinal_track_counter,
            ordinal_disc_number: ordinal_disc_counter,
            harvest: &harvest_data,
            source: entry,
            album_source: &album_src,
            album_root,
            library_root,
        };

        for key in &track_reg {
            if let Some(val) = resolve::resolve_track_standard(key, &track_ctx) {
                track_obj.insert(key.clone(), val);
            }
        }

        let mut h_item = serde_json::to_value(&harvest_data)?;
        if let Ok(rel) = harvest_data.path.strip_prefix(album_root) {
            h_item["track_path"] = json!(rel.to_string_lossy());
        }
        harvested_cache.push(h_item);
        final_tracks.push(Value::Object(track_obj));
    }

    let album_ctx = AlbumContext {
        source: &album_src,
        tracks: &final_tracks,
        album_root,
        library_root,
        meta_hash: &metadata_hash,
        meta_mtime: metadata_mtime,
        cover_hash: &cover_hash,
        cover_path: cover_path.as_deref(),
        cover_image: loaded_image.as_ref(),
    };

    let mut final_album = serde_json::Map::new();
    for key in &album_reg {
        if let Some(val) = resolve::resolve_album_standard(key, &album_ctx) {
            final_album.insert(key.clone(), val);
        }
    }

    Ok(json!({
        "album": Value::Object(final_album),
        "tracks": final_tracks,
        "ctx": {
            "config": config,
            "metadata": metadata_json,
            "harvest": harvested_cache,
            "paths": {
                "album_root": album_root.to_string_lossy(),
                "project_root": project_root.to_string_lossy(),
                "library_root": library_root.to_string_lossy(),
            }
        }
    }))
}

fn parse_tag_int(val: Option<&String>) -> u32 {
    val.and_then(|s| s.split('/').next()).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0)
}

fn resolve_cover_info(root: &Path) -> (Option<String>, String) {
    for c in ["cover.jpg", "cover.png", "folder.jpg", "front.jpg"] {
        let p = root.join(c);
        if p.exists() {
            if let Ok(mut file) = File::open(&p) {
                let mut hasher = Xxh64::new(0);
                let mut buffer = [0; 8192];
                loop {
                    match file.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => hasher.write(&buffer[..n]),
                        Err(_) => return (Some(c.to_string()), String::new()),
                    }
                }
                let hash = hasher.finish();
                return (Some(c.to_string()), URL_SAFE_NO_PAD.encode(hash.to_be_bytes()));
            }
        }
    }
    (None, String::new())
}
