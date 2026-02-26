use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use image::{DynamicImage, GenericImageView};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::SystemTime;

use crate::compile::resolve::{self, AlbumContext, TrackContext};
use crate::compile::scan;
use crate::expand_path;
use crate::harvest;

pub fn build(
    album_root: &Path,
    project_root: &Path,
    config: &Value,
    gen_cfg: &Value,
    active_flags: &[String],
    _no_extensions: bool,
) -> Result<(Value, bool)> {
    let metadata_path = album_root.join("metadata.toml");
    let (metadata_mtime, metadata_hash) = get_file_stats(&metadata_path)?;

    let content = std::fs::read_to_string(&metadata_path)?;
    let metadata_toml: toml::Value = toml::from_str(&content)?;
    let metadata_json = normalize_json_keys(serde_json::to_value(metadata_toml)?);

    let (cover_path, cover_hash, cover_mtime, cover_byte_size) = resolve_cover_info(album_root);
    let loaded_image =
        load_or_create_thumbnail(config, album_root, cover_path.as_deref(), &cover_hash);

    let exts = get_supported_extensions(gen_cfg);
    let audio_files = scan::scan_audio_files(album_root, &exts);

    let track_entries = metadata_json
        .get("tracks")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Missing [[tracks]] in metadata.toml"))?;

    if audio_files.len() != track_entries.len() {
        return Err(anyhow!(
            "Manifest Mismatch: {} files vs {} entries",
            audio_files.len(),
            track_entries.len()
        ));
    }

    let registry = config
        .get("compiler_registry")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("Missing [compiler_registry]"))?;

    let library_root = Path::new(
        config
            .get("storage")
            .and_then(|s| s.get("library_root"))
            .and_then(Value::as_str)
            .unwrap_or("."),
    );

    let empty_album_source = json!({});
    let album_source_data = metadata_json.get("album").unwrap_or(&empty_album_source);

    let (final_tracks, harvested_cache, mut requires_external) = process_tracks(
        album_root,
        library_root,
        &audio_files,
        track_entries,
        album_source_data,
        registry,
    )?;

    let album_ctx = AlbumContext {
        source: album_source_data,
        tracks: &final_tracks,
        album_root,
        library_root,
        meta_hash: &metadata_hash,
        meta_mtime: metadata_mtime,
        cover_hash: &cover_hash,
        cover_path: cover_path.as_deref(),
        cover_mtime,
        cover_byte_size,
        cover_image: loaded_image.as_ref(),
    };

    let (album_obj, album_requires_external) =
        build_album_object(&album_ctx, registry, requires_external);

    requires_external = album_requires_external;

    let final_json = json!({
        "album": album_obj,
        "tracks": final_tracks,
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
    });

    Ok((final_json, requires_external))
}

fn get_file_stats(path: &Path) -> Result<(u64, String)> {
    let meta = std::fs::metadata(path)?;
    let mtime = meta.modified()?.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    std::io::copy(&mut reader, &mut hasher)?;
    Ok((mtime, format!("{:x}", hasher.finalize())))
}

fn get_supported_extensions(gen_cfg: &Value) -> Vec<&str> {
    gen_cfg
        .get("supported_extensions")
        .and_then(Value::as_array)
        .map_or_else(|| vec![".flac"], |arr| arr.iter().filter_map(Value::as_str).collect())
}

fn load_or_create_thumbnail(
    config: &Value,
    album_root: &Path,
    cover_path: Option<&str>,
    cover_hash: &str,
) -> Option<DynamicImage> {
    let storage = config.get("storage")?;
    let dir_str = storage.get("thumbnail_cache_folder").and_then(Value::as_str)?;
    let cp = cover_path?;
    if cover_hash.is_empty() {
        return None;
    }

    let thumb_dir = expand_path(dir_str);
    let thumb_path = thumb_dir.join(format!("{cover_hash}.png"));

    if !thumb_path.exists() {
        if let Ok(img) = image::open(album_root.join(cp)) {
            let (w, h) = img.dimensions();
            let side = std::cmp::min(w, h);
            let square = img.crop_imm((w - side) / 2, (h - side) / 2, side, side);
            let final_thumb = square.resize(200, 200, image::imageops::FilterType::Lanczos3);
            let _ = std::fs::create_dir_all(&thumb_dir);
            let _ = final_thumb.save(&thumb_path);
            return Some(final_thumb);
        }
    } else if let Ok(img) = image::open(&thumb_path) {
        return Some(img);
    }
    None
}

fn process_tracks(
    album_root: &Path,
    library_root: &Path,
    audio_files: &[std::path::PathBuf],
    track_entries: &[Value],
    metadata_album_source: &Value,
    registry: &serde_json::Map<String, Value>,
) -> Result<(Vec<Value>, Vec<Value>, bool)> {
    let mut harvested_spine = Vec::new();
    for path in audio_files {
        harvested_spine.push(harvest::harvest_file(path)?);
    }

    harvested_spine.sort_by(|a, b| {
        let a_disc = parse_tag_int(a.tags.get("DISCNUMBER"));
        let b_disc = parse_tag_int(b.tags.get("DISCNUMBER"));
        if a_disc != b_disc {
            return a_disc.cmp(&b_disc);
        }
        let a_track = parse_tag_int(a.tags.get("TRACKNUMBER"));
        let b_track = parse_tag_int(b.tags.get("TRACKNUMBER"));
        if a_track != b_track {
            return a_track.cmp(&b_track);
        }
        a.path.cmp(&b.path)
    });

    let mut requires_external = false;
    let mut final_tracks = Vec::new();
    let mut harvested_cache = Vec::new();
    let mut current_physical_disc = None;
    let mut o_disc = 0;
    let mut o_track = 0;

    for (idx, h_data) in harvested_spine.into_iter().enumerate() {
        let p_disc = parse_tag_int(h_data.tags.get("DISCNUMBER"));
        if Some(p_disc) == current_physical_disc {
            o_track += 1;
        } else {
            current_physical_disc = Some(p_disc);
            o_disc += 1;
            o_track = 1;
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
        track_info
            .insert("track_path".to_string(), json!(resolve::rel_path(&h_data.path, album_root)));
        track_info.insert(
            "track_library_path".to_string(),
            json!(resolve::rel_path(&h_data.path, library_root)),
        );
        track_info.insert("track_duration".to_string(), json!(h_data.physics.duration_ms));
        track_info.insert(
            "track_duration_time".to_string(),
            json!(resolve::format_ms(h_data.physics.duration_ms)),
        );
        track_info.insert("encoding".to_string(), json!(h_data.physics.format));
        track_info.insert("sample_rate".to_string(), json!(h_data.physics.sample_rate));
        track_info
            .insert("bits_per_sample".to_string(), json!(h_data.physics.bit_depth.unwrap_or(0)));
        track_info.insert("channels".to_string(), json!(h_data.physics.channels));
        track_info.insert("track_mtime".to_string(), json!(h_data.physics.mtime));
        track_info.insert("track_byte_size".to_string(), json!(h_data.physics.file_size));
        track_info.insert("lyrics_path".to_string(), json!(""));

        track_obj.insert("info".to_string(), Value::Object(track_info));
        track_obj.insert(
            "TITLE".to_string(),
            track_entries_source.get("title").cloned().unwrap_or_else(|| {
                resolve::resolve_track_key("title", &track_ctx).unwrap_or(Value::Null)
            }),
        );
        track_obj.insert(
            "ARTIST".to_string(),
            track_entries_source.get("artist").cloned().unwrap_or_else(|| {
                resolve::resolve_track_key("artist", &track_ctx).unwrap_or(Value::Null)
            }),
        );

        let t_num = to_strict_u32(track_entries_source.get("tracknumber"))
            .max(to_strict_u32(resolve::resolve_track_key("tracknumber", &track_ctx).as_ref()));
        let d_num = to_strict_u32(track_entries_source.get("discnumber"))
            .max(to_strict_u32(resolve::resolve_track_key("discnumber", &track_ctx).as_ref()));

        track_obj.insert("TRACKNUMBER".to_string(), json!(t_num));
        track_obj.insert("DISCNUMBER".to_string(), json!(d_num));

        let mut track_tags = serde_json::Map::new();
        for (key, meta) in registry {
            if meta.get("level").and_then(Value::as_str) != Some("tracks") {
                continue;
            }

            let val = track_entries_source.get(key).map_or_else(
                || {
                    if meta.get("provider").and_then(Value::as_str) == Some("extension") {
                        requires_external = true;
                        json!(null)
                    } else {
                        resolve::resolve_track_key(key, &track_ctx).unwrap_or(Value::Null)
                    }
                },
                Clone::clone,
            );

            track_tags.insert(key.to_uppercase(), val);
        }
        track_obj.insert("tags".to_string(), Value::Object(track_tags));

        let mut h_item = serde_json::to_value(&h_data)?;
        h_item["track_path"] = json!(resolve::rel_path(&h_data.path, album_root));
        harvested_cache.push(h_item);
        final_tracks.push(Value::Object(track_obj));
    }
    Ok((final_tracks, harvested_cache, requires_external))
}

fn build_album_object(
    ctx: &AlbumContext,
    registry: &serde_json::Map<String, Value>,
    ext_flag: bool,
) -> (Value, bool) {
    let mut album_obj = serde_json::Map::new();
    let mut album_info = serde_json::Map::new();
    album_info.insert(
        "album_path".to_string(),
        json!(resolve::rel_path(ctx.album_root, ctx.library_root)),
    );
    album_info.insert("unix_added".to_string(), json!(resolve::resolve_album_info_unix_added(ctx)));
    album_info
        .insert("album_duration".to_string(), json!(resolve::resolve_album_info_duration_ms(ctx)));
    album_info.insert(
        "album_duration_time".to_string(),
        json!(resolve::format_ms(resolve::resolve_album_info_duration_ms(ctx))),
    );
    album_info.insert("total_discs".to_string(), json!(resolve::calculate_total_discs(ctx.tracks)));
    album_info.insert("total_tracks".to_string(), json!(ctx.tracks.len()));
    album_info.insert("metadata_toml_hash".to_string(), json!(ctx.meta_hash));
    album_info.insert("metadata_toml_mtime".to_string(), json!(ctx.meta_mtime));
    album_info.insert("file_tag_subset_match".to_string(), json!(false));
    album_info
        .insert("cover_path".to_string(), json!(ctx.cover_path.unwrap_or("default_cover.png")));
    album_info.insert("cover_hash".to_string(), json!(ctx.cover_hash));
    album_info.insert("cover_mtime".to_string(), json!(ctx.cover_mtime));
    album_info.insert("cover_byte_size".to_string(), json!(ctx.cover_byte_size));

    album_obj.insert("info".to_string(), Value::Object(album_info));
    album_obj.insert(
        "ALBUM".to_string(),
        ctx.source
            .get("album")
            .cloned()
            .unwrap_or_else(|| resolve::resolve_album_key("album", ctx).unwrap_or(Value::Null)),
    );
    album_obj.insert(
        "ALBUMARTIST".to_string(),
        ctx.source.get("albumartist").cloned().unwrap_or_else(|| {
            resolve::resolve_album_key("albumartist", ctx).unwrap_or(Value::Null)
        }),
    );
    album_obj.insert(
        "DATE".to_string(),
        ctx.source
            .get("date")
            .cloned()
            .unwrap_or_else(|| resolve::resolve_album_key("date", ctx).unwrap_or(Value::Null)),
    );
    album_obj.insert(
        "GENRE".to_string(),
        ctx.source
            .get("genre")
            .cloned()
            .unwrap_or_else(|| resolve::resolve_album_key("genre", ctx).unwrap_or(Value::Null)),
    );
    album_obj.insert(
        "COMMENT".to_string(),
        ctx.source
            .get("comment")
            .cloned()
            .unwrap_or_else(|| resolve::resolve_album_key("comment", ctx).unwrap_or(Value::Null)),
    );
    album_obj.insert(
        "ORIGINAL_YYYY_MM".to_string(),
        ctx.source.get("original_yyyy_mm").cloned().unwrap_or_else(|| {
            resolve::resolve_album_key("original_yyyy_mm", ctx).unwrap_or(Value::Null)
        }),
    );
    album_obj.insert(
        "RELEASE_YYYY_MM".to_string(),
        ctx.source.get("release_yyyy_mm").cloned().unwrap_or_else(|| {
            resolve::resolve_album_key("release_yyyy_mm", ctx).unwrap_or(Value::Null)
        }),
    );

    let mut album_tags = serde_json::Map::new();
    let mut ext_needed = ext_flag;
    for (key, meta) in registry {
        if meta.get("level").and_then(Value::as_str) != Some("album") {
            continue;
        }

        let val = ctx.source.get(key).map_or_else(
            || {
                if meta.get("provider").and_then(Value::as_str) == Some("extension") {
                    ext_needed = true;
                    json!(null)
                } else {
                    resolve::resolve_album_key(key, ctx).unwrap_or(Value::Null)
                }
            },
            Clone::clone,
        );

        album_tags.insert(key.to_uppercase(), val);
    }
    album_obj.insert("tags".to_string(), Value::Object(album_tags));
    (Value::Object(album_obj), ext_needed)
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

fn to_strict_u32(v: Option<&Value>) -> u32 {
    match v {
        Some(Value::Number(n)) => u32::try_from(n.as_u64().unwrap_or(0)).unwrap_or(0),
        Some(Value::String(s)) => s.split('/').next().unwrap_or("0").parse().unwrap_or(0),
        _ => 0,
    }
}

fn resolve_cover_info(root: &Path) -> (Option<String>, String, u64, u64) {
    let candidates = ["cover.jpg", "cover.png", "folder.jpg", "front.jpg"];
    for c in candidates {
        let p = root.join(c);
        if let Ok(m) = std::fs::metadata(&p) {
            let mtime =
                m.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let size = m.len();
            let mut h = sha2::Sha256::new();
            h.update(mtime.to_be_bytes());
            h.update(size.to_be_bytes());
            return (Some(c.to_string()), URL_SAFE_NO_PAD.encode(&h.finalize()[..8]), mtime, size);
        }
    }
    (None, String::new(), 0, 0)
}
