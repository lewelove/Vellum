use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::SystemTime;
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
    active_flags: &[String],
    _no_extensions: bool,
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
    let metadata_json = normalize_json_keys(serde_json::to_value(metadata_toml)?);

    let (cover_path, cover_hash, cover_mtime, cover_byte_size) = resolve_cover_info(album_root);
    let mut loaded_image: Option<DynamicImage> = None;

    let storage = config.get("storage").ok_or_else(|| anyhow!("Missing [storage]"))?;
    if let (Some(dir_str), Some(cp), false) = (storage.get("thumbnail_cache_folder").and_then(|v| v.as_str()), &cover_path, cover_hash.is_empty()) {
        let thumb_dir = expand_path(dir_str);
        let thumb_path = thumb_dir.join(format!("{}.png", cover_hash));
        if !thumb_path.exists() {
            if let Ok(img) = image::open(album_root.join(cp)) {
                let (w, h) = img.dimensions();
                let side = std::cmp::min(w, h);
                let square = img.crop_imm((w - side) / 2, (h - side) / 2, side, side);
                let final_thumb = square.resize(200, 200, image::imageops::FilterType::Lanczos3);
                let _ = std::fs::create_dir_all(&thumb_dir);
                let _ = final_thumb.save(&thumb_path);
                loaded_image = Some(final_thumb);
            }
        } else if let Ok(img) = image::open(&thumb_path) {
            loaded_image = Some(img);
        }
    }

    let exts_raw = gen_cfg.get("supported_extensions").and_then(|v| v.as_array());
    let exts: Vec<&str> = exts_raw.map(|arr| arr.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_else(|| vec![".flac"]);
    let audio_files = scan::scan_audio_files(album_root, &exts);

    let track_entries = metadata_json.get("tracks").and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing [[tracks]] in metadata.toml"))?;

    if audio_files.len() != track_entries.len() {
        return Err(anyhow!("Manifest Mismatch: {} files vs {} entries", audio_files.len(), track_entries.len()));
    }

    let registry = config.get("compiler_registry").and_then(|v| v.as_object())
        .ok_or_else(|| anyhow!("Missing [compiler_registry]"))?;

    let library_root = Path::new(storage.get("library_root").and_then(|v| v.as_str()).unwrap_or("."));

    let mut harvested_spine = Vec::new();
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

    let mut requires_python = false;
    let mut final_tracks = Vec::new();
    let mut harvested_cache = Vec::new();
    let mut current_physical_disc = None;
    let mut o_disc = 0;
    let mut o_track = 0;

    let empty_obj = json!({});
    let metadata_album_source = metadata_json.get("album").unwrap_or(&empty_obj);

    for (idx, h_data) in harvested_spine.into_iter().enumerate() {
        let p_disc = parse_tag_int(h_data.tags.get("DISCNUMBER"));
        if Some(p_disc) != current_physical_disc {
            current_physical_disc = Some(p_disc);
            o_disc += 1;
            o_track = 1;
        } else {
            o_track += 1;
        }

        let track_entries_source = &track_entries[idx];
        let track_ctx = TrackContext {
            ordinal_track_number: o_track,
            ordinal_disc_number: o_disc,
            _harvest: &h_data,
            source: track_entries_source,
            album_source: metadata_album_source,
            _album_root: album_root,
            _library_root: library_root,
        };

        let mut track_obj = serde_json::Map::new();
        let mut track_info = serde_json::Map::new();
        track_info.insert("track_path".to_string(), json!(resolve::rel_path(&h_data.path, album_root)));
        track_info.insert("track_library_path".to_string(), json!(resolve::rel_path(&h_data.path, library_root)));
        track_info.insert("track_duration".to_string(), json!(h_data.physics.duration_ms));
        track_info.insert("track_duration_time".to_string(), json!(resolve::format_ms(h_data.physics.duration_ms)));
        track_info.insert("encoding".to_string(), json!(h_data.physics.format));
        track_info.insert("sample_rate".to_string(), json!(h_data.physics.sample_rate));
        track_info.insert("bits_per_sample".to_string(), json!(h_data.physics.bit_depth.unwrap_or(0)));
        track_info.insert("channels".to_string(), json!(h_data.physics.channels));
        track_info.insert("track_mtime".to_string(), json!(h_data.physics.mtime));
        track_info.insert("track_byte_size".to_string(), json!(h_data.physics.file_size));
        track_info.insert("lyrics_path".to_string(), json!("")); 

        track_obj.insert("info".to_string(), Value::Object(track_info));

        let core_track_tags = [
            "TITLE",
            "ARTIST",
            "TRACKNUMBER",
            "DISCNUMBER"
        ];

        for key in core_track_tags {
            let val = track_entries_source.get(&key.to_lowercase())
                .cloned()
                .unwrap_or_else(|| resolve::resolve_track_key(&key.to_lowercase(), &track_ctx).unwrap_or(json!(null)));
            track_obj.insert(key.to_string(), val);
        }

        for (key, meta) in registry {
            if meta.get("level").and_then(|v| v.as_str()) != Some("tracks") { continue; }
            
            let val = if let Some(user_val) = track_entries_source.get(key) {
                user_val.clone()
            } else if meta.get("provider").and_then(|v| v.as_str()) == Some("extension") {
                requires_python = true;
                json!(null)
            } else {
                resolve::resolve_track_key(key, &track_ctx).unwrap_or(json!(null))
            };
            
            track_obj.insert(key.to_uppercase(), val);
        }

        let mut h_item = serde_json::to_value(&h_data)?;
        h_item["track_path"] = json!(resolve::rel_path(&h_data.path, album_root));
        harvested_cache.push(h_item);
        final_tracks.push(Value::Object(track_obj));
    }

    let album_ctx = AlbumContext {
        source: metadata_album_source,
        tracks: &final_tracks,
        _album_root: album_root,
        _library_root: library_root,
        _meta_hash: &metadata_hash,
        _meta_mtime: metadata_mtime,
        _cover_hash: &cover_hash,
        _cover_path: cover_path.as_deref(),
        _cover_mtime: cover_mtime,
        _cover_byte_size: cover_byte_size,
        cover_image: loaded_image.as_ref(),
    };

    let mut album_obj = serde_json::Map::new();
    let mut album_info = serde_json::Map::new();
    album_info.insert("album_path".to_string(), json!(resolve::rel_path(album_root, library_root)));
    album_info.insert("unix_added".to_string(), json!(resolve::resolve_album_info_unix_added(&album_ctx)));
    album_info.insert("album_duration".to_string(), json!(resolve::resolve_album_info_duration_ms(&album_ctx)));
    album_info.insert("album_duration_time".to_string(), json!(resolve::format_ms(resolve::resolve_album_info_duration_ms(&album_ctx))));
    album_info.insert("total_discs".to_string(), json!(resolve::calculate_total_discs(&final_tracks)));
    album_info.insert("total_tracks".to_string(), json!(final_tracks.len()));
    album_info.insert("metadata_toml_hash".to_string(), json!(metadata_hash));
    album_info.insert("metadata_toml_mtime".to_string(), json!(metadata_mtime));
    album_info.insert("file_tag_subset_match".to_string(), json!(false));
    album_info.insert("cover_path".to_string(), json!(cover_path.clone().unwrap_or_else(|| "default_cover.png".to_string())));
    album_info.insert("cover_hash".to_string(), json!(cover_hash));
    album_info.insert("cover_mtime".to_string(), json!(cover_mtime));
    album_info.insert("cover_byte_size".to_string(), json!(cover_byte_size));

    album_obj.insert("info".to_string(), Value::Object(album_info));

    let core_album_tags = [
        "ALBUM",
        "ALBUMARTIST",
        "DATE"
    ];

    for key in core_album_tags {
        let val = metadata_album_source.get(&key.to_lowercase())
            .cloned()
            .unwrap_or_else(|| resolve::resolve_album_key(&key.to_lowercase(), &album_ctx).unwrap_or(json!(null)));
        album_obj.insert(key.to_string(), val);
    }

    for (key, meta) in registry {
        if meta.get("level").and_then(|v| v.as_str()) != Some("album") { continue; }
        
        let val = if let Some(user_val) = metadata_album_source.get(key) {
            user_val.clone()
        } else if meta.get("provider").and_then(|v| v.as_str()) == Some("extension") {
            requires_python = true;
            json!(null)
        } else {
            resolve::resolve_album_key(key, &album_ctx).unwrap_or(json!(null))
        };
        
        album_obj.insert(key.to_uppercase(), val);
    }

    Ok(json!({
        "album": Value::Object(album_obj),
        "tracks": final_tracks,
        "requires_python": requires_python,
        "ctx": {
            "config": config,
            "active_flags": active_flags,
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

fn normalize_json_keys(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, val) in map {
                new_map.insert(k.to_lowercase(), normalize_json_keys(val));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize_json_keys).collect()),
        _ => v,
    }
}

fn parse_tag_int(val: Option<&String>) -> u32 {
    val.and_then(|s| s.split('/').next()).and_then(|s| s.parse().ok()).unwrap_or(0)
}

fn resolve_cover_info(root: &Path) -> (Option<String>, String, u64, u64) {
    let candidates = ["cover.jpg", "cover.png", "folder.jpg", "front.jpg"];
    for c in candidates {
        let p = root.join(c);
        if let Ok(m) = std::fs::metadata(&p) {
            let mtime = m.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let size = m.len();
            let mut h = sha2::Sha256::new();
            h.update(mtime.to_be_bytes());
            h.update(size.to_be_bytes());
            return (Some(c.to_string()), URL_SAFE_NO_PAD.encode(&h.finalize()[..8]), mtime, size);
        }
    }
    (None, String::new(), 0, 0)
}
