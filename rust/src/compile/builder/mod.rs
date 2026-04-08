pub mod assets;
pub mod context;
pub mod scan;

use crate::compile::builder::context::{AlbumContext, TrackContext};
use crate::compile::resolvers;
use crate::expand_path;
use crate::harvest;
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use sha2::Digest;
use std::collections::HashSet;
use std::path::Path;

pub fn build(
    album_root: &Path,
    project_root: &Path,
    config: &Value,
    manifest_cfg: &Value,
    _active_flags: &[String],
) -> Result<Value> {
    let metadata_path = album_root.join("metadata.toml");
    let meta = std::fs::metadata(&metadata_path)?;
    let meta_mtime = meta
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let meta_hash = format!("{:x}", sha2::Sha256::digest(std::fs::read(&metadata_path)?));

    let content = std::fs::read_to_string(&metadata_path)?;
    let metadata_json = normalize_keys(serde_json::to_value(toml::from_str::<toml::Value>(
        &content,
    )?)?);

    let (c_path, c_hash, c_mtime, c_size) = assets::resolve_cover_info(album_root);
    let loaded_image =
        assets::load_or_create_thumbnail(config, album_root, c_path.as_deref(), &c_hash);

    let exts: Vec<&str> = manifest_cfg
        .get("supported_extensions")
        .and_then(Value::as_array)
        .map_or_else(
            || vec![".flac"],
            |arr| arr.iter().filter_map(Value::as_str).collect(),
        );

    let audio_files = scan::scan_audio_files(album_root, &exts);
    let track_entries = metadata_json
        .get("tracks")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Missing tracks in metadata.toml"))?;

    if audio_files.len() != track_entries.len() {
        return Err(anyhow!(
            "Track count mismatch in {}: Found {} files but {} [[tracks]] entries",
            album_root.display(),
            audio_files.len(),
            track_entries.len()
        ));
    }

    validate_track_indices(track_entries, album_root)?;

    let lib_root_raw = config
        .get("storage")
        .and_then(|s| s.get("library_root"))
        .and_then(Value::as_str)
        .unwrap_or(".");
    let library_root = expand_path(lib_root_raw)
        .canonicalize()
        .unwrap_or_else(|_| expand_path(lib_root_raw));

    let mut registry = config
        .get("compiler")
        .and_then(|c| c.get("keys"))
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("Missing registry keys"))?
        .clone();

    let local_config_path = album_root.join("config.toml");
    if local_config_path.exists() {
        if let Ok(local_content) = std::fs::read_to_string(&local_config_path) {
            if let Ok(local_toml) = toml::from_str::<toml::Value>(&local_content) {
                if let Ok(local_json) = serde_json::to_value(local_toml) {
                    if let Some(local_keys) = local_json
                        .get("compiler")
                        .and_then(|c| c.get("keys"))
                        .and_then(Value::as_object)
                    {
                        for (k, v) in local_keys {
                            if let Some(existing) = registry.get_mut(k) {
                                if let (Some(existing_obj), Some(new_obj)) = (existing.as_object_mut(), v.as_object()) {
                                    for (nk, nv) in new_obj {
                                        existing_obj.insert(nk.clone(), nv.clone());
                                    }
                                } else {
                                    registry.insert(k.clone(), v.clone());
                                }
                            } else {
                                registry.insert(k.clone(), v.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    let empty_obj = json!({});
    let album_source = metadata_json.get("album").unwrap_or(&empty_obj);

    validate_album_level_keys(album_source, track_entries, &registry, album_root)?;

    let (final_tracks, harvested_cache) = process_tracks(
        audio_files,
        track_entries,
        album_source,
        album_root,
        &library_root,
        &registry,
    )?;

    let album_ctx = AlbumContext {
        source: album_source,
        tracks: &final_tracks,
        album_root,
        library_root: &library_root,
        meta_hash: &meta_hash,
        meta_mtime,
        cover_hash: &c_hash,
        cover_path: c_path.as_deref(),
        cover_mtime: c_mtime,
        cover_byte_size: c_size,
        cover_image: loaded_image.as_ref(),
    };

    let album_obj = build_album(&album_ctx, &registry);

    let final_json = json!({
        "album": album_obj,
        "tracks": final_tracks,
        "ctx": {
            "config": config,
            "registry": registry,
            "metadata": metadata_json,
            "harvest": harvested_cache,
            "paths": {
                "album_root": album_root.to_string_lossy(),
                "project_root": project_root.to_string_lossy(),
                "library_root": library_root.to_string_lossy()
            }
        }
    });

    Ok(final_json)
}

fn validate_album_level_keys(
    album_source: &Value,
    track_entries: &[Value],
    registry: &serde_json::Map<String, Value>,
    album_root: &Path,
) -> Result<()> {
    for (key, meta) in registry {
        if meta.get("level").and_then(Value::as_str) != Some("album") {
            continue;
        }
        
        let key_lower = key.to_lowercase();
        let mut seen_values: Vec<(Value, String)> = Vec::new();
        
        let check_val = |v: &Value| -> bool {
            match v {
                Value::Null => false,
                Value::String(s) => !s.trim().is_empty(),
                Value::Array(a) => !a.is_empty(),
                _ => true,
            }
        };

        if let Some(v) = album_source.get(&key_lower).filter(|v| check_val(v)) {
            seen_values.push((v.clone(), "album section".to_string()));
        }

        for (idx, track) in track_entries.iter().enumerate() {
            if let Some(v) = track.get(&key_lower).filter(|v| check_val(v)) {
                if let Some((first_val, source_name)) = seen_values.first() {
                    if v != first_val {
                        return Err(anyhow::anyhow!(
                            "Validation failed in {}: key '{}' is defined as level=\"album\", but conflicting values were found ('{}' in {} vs '{}' in track {}). All tracks must share the same value for album-level keys.",
                            album_root.display(),
                            key,
                            first_val,
                            source_name,
                            v,
                            idx + 1
                        ));
                    }
                } else {
                    seen_values.push((v.clone(), format!("track {}", idx + 1)));
                }
            }
        }
    }
    Ok(())
}

fn process_tracks(
    audio_files: Vec<std::path::PathBuf>,
    track_entries: &[Value],
    album_source: &Value,
    album_root: &Path,
    library_root: &Path,
    registry: &serde_json::Map<String, Value>,
) -> Result<(Vec<Value>, Vec<Value>)> {
    let mut harvested_spine = Vec::new();
    for path in audio_files {
        harvested_spine.push(harvest::harvest_file(&path)?);
    }

    let total_discs = u32::try_from(
        track_entries
            .iter()
            .filter_map(|t| {
                t.get("discnumber").and_then(|v| match v {
                    Value::Number(n) => n.as_u64(),
                    Value::String(s) => s.parse::<u64>().ok(),
                    _ => None,
                })
            })
            .max()
            .unwrap_or(1),
    )
    .unwrap_or(u32::MAX);

    let mut final_tracks = Vec::new();
    let mut harvested_cache = Vec::new();

    for (idx, h_data) in harvested_spine.into_iter().enumerate() {
        let t_ctx = TrackContext {
            ordinal_track_number: u32::try_from(idx + 1).unwrap_or(u32::MAX),
            ordinal_disc_number: 1,
            harvest: &h_data,
            source: &track_entries[idx],
            album_source,
            album_root,
            library_root,
        };

        let t_obj = build_track(&t_ctx, total_discs, registry);
        final_tracks.push(t_obj);
        harvested_cache.push(serde_json::to_value(h_data)?);
    }

    Ok((final_tracks, harvested_cache))
}

fn validate_track_indices(entries: &[Value], root: &Path) -> Result<()> {
    let mut seen_ids = HashSet::new();
    for (idx, entry) in entries.iter().enumerate() {
        let t = entry
            .get("tracknumber")
            .and_then(|v| match v {
                Value::Number(n) => n.as_u64().and_then(|i| u32::try_from(i).ok()),
                Value::String(s) => s.parse::<u32>().ok(),
                _ => None,
            })
            .unwrap_or_else(|| u32::try_from(idx + 1).unwrap_or(u32::MAX));

        let d = entry
            .get("discnumber")
            .and_then(|v| match v {
                Value::Number(n) => n.as_u64().and_then(|i| u32::try_from(i).ok()),
                Value::String(s) => s.parse::<u32>().ok(),
                _ => None,
            })
            .unwrap_or(1);

        if !seen_ids.insert((d, t)) {
            return Err(anyhow!(
                "Collision in {}: Multiple tracks assigned to Disc {}, Track {}",
                root.display(),
                d,
                t
            ));
        }
    }
    Ok(())
}

fn normalize_keys(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, val) in map {
                new_map.insert(k.to_lowercase(), normalize_keys(val));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize_keys).collect()),
        _ => v,
    }
}

fn construct_track_info(ctx: &TrackContext, total_discs: u32) -> Value {
    let mut info = serde_json::Map::new();

    let track_number = ctx
        .source
        .get("tracknumber")
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64().and_then(|i| u32::try_from(i).ok()),
            Value::String(s) => s.parse::<u32>().ok(),
            _ => None,
        })
        .unwrap_or(ctx.ordinal_track_number);

    let disc_number = ctx
        .source
        .get("discnumber")
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64().and_then(|i| u32::try_from(i).ok()),
            Value::String(s) => s.parse::<u32>().ok(),
            _ => None,
        })
        .unwrap_or(ctx.ordinal_disc_number);

    let lyrics_path = ctx
        .source
        .get("lyrics_path")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            resolvers::native::resolve_lyrics_path(
                ctx.album_root,
                track_number,
                disc_number,
                total_discs,
            )
        });

    info.insert(
        "track_path".to_string(),
        json!(resolvers::native::rel_path(
            &ctx.harvest.path,
            ctx.album_root
        )),
    );
    info.insert(
        "track_library_path".to_string(),
        json!(resolvers::native::rel_path(
            &ctx.harvest.path,
            ctx.library_root
        )),
    );
    info.insert(
        "track_duration".to_string(),
        json!(ctx.harvest.physics.duration_ms),
    );
    info.insert(
        "track_duration_time".to_string(),
        json!(resolvers::standard::format_ms(
            ctx.harvest.physics.duration_ms
        )),
    );
    info.insert("encoding".to_string(), json!(ctx.harvest.physics.format));
    info.insert(
        "sample_rate".to_string(),
        json!(ctx.harvest.physics.sample_rate),
    );
    info.insert(
        "bits_per_sample".to_string(),
        json!(ctx.harvest.physics.bit_depth.unwrap_or(0)),
    );
    info.insert("channels".to_string(), json!(ctx.harvest.physics.channels));
    info.insert("track_mtime".to_string(), json!(ctx.harvest.physics.mtime));
    info.insert(
        "track_size".to_string(),
        json!(ctx.harvest.physics.file_size),
    );
    info.insert(
        "lyrics_path".to_string(),
        json!(lyrics_path.unwrap_or_default()),
    );

    Value::Object(info)
}

fn build_track(
    ctx: &TrackContext,
    total_discs: u32,
    registry: &serde_json::Map<String, Value>,
) -> Value {
    let mut obj = serde_json::Map::new();

    let track_number = ctx
        .source
        .get("tracknumber")
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64().and_then(|i| u32::try_from(i).ok()),
            Value::String(s) => s.parse::<u32>().ok(),
            _ => None,
        })
        .unwrap_or(ctx.ordinal_track_number);

    let disc_number = ctx
        .source
        .get("discnumber")
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64().and_then(|i| u32::try_from(i).ok()),
            Value::String(s) => s.parse::<u32>().ok(),
            _ => None,
        })
        .unwrap_or(ctx.ordinal_disc_number);

    obj.insert("info".to_string(), construct_track_info(ctx, total_discs));

    obj.insert("TITLE".to_string(), resolvers::resolve_top_level_track_key("TITLE", ctx));
    obj.insert("ARTIST".to_string(), resolvers::resolve_top_level_track_key("ARTIST", ctx));
    obj.insert("TRACKNUMBER".to_string(), json!(track_number));
    obj.insert("DISCNUMBER".to_string(), json!(disc_number));

    let mut tags = serde_json::Map::new();
    for (key, meta) in registry {
        let key_lower = key.to_lowercase();
        if key_lower == "title" || key_lower == "artist" || key_lower == "tracknumber" || key_lower == "discnumber" {
            continue;
        }
        let val = resolvers::resolve_track_key(key, meta, ctx).unwrap_or(Value::Null);
        tags.insert(key.to_uppercase(), val);
    }
    obj.insert("tags".to_string(), Value::Object(tags));
    Value::Object(obj)
}

fn construct_album_info(ctx: &AlbumContext) -> Value {
    let mut info = serde_json::Map::new();
    let dur: u64 = ctx
        .tracks
        .iter()
        .filter_map(|t| {
            t.get("info")
                .and_then(|i| i.get("track_duration"))
                .and_then(Value::as_u64)
        })
        .sum();

    info.insert(
        "album_path".to_string(),
        json!(resolvers::native::rel_path(
            ctx.album_root,
            ctx.library_root
        )),
    );
    info.insert(
        "unix_added".to_string(),
        json!(resolvers::native::resolve_album_info_unix_added(ctx, "")),
    );
    info.insert("album_duration".to_string(), json!(dur));
    info.insert(
        "album_duration_time".to_string(),
        json!(resolvers::standard::format_ms(dur)),
    );
    info.insert(
        "total_discs".to_string(),
        json!(resolvers::native::calculate_total_discs(ctx.tracks)),
    );
    info.insert("total_tracks".to_string(), json!(ctx.tracks.len()));
    info.insert("metadata_toml_hash".to_string(), json!(ctx.meta_hash));
    info.insert("metadata_toml_mtime".to_string(), json!(ctx.meta_mtime));
    info.insert(
        "cover_path".to_string(),
        json!(ctx.cover_path.unwrap_or("default_cover.png")),
    );
    info.insert("cover_hash".to_string(), json!(ctx.cover_hash));
    info.insert("cover_mtime".to_string(), json!(ctx.cover_mtime));
    info.insert("cover_byte_size".to_string(), json!(ctx.cover_byte_size));

    Value::Object(info)
}

fn build_album(
    ctx: &AlbumContext,
    registry: &serde_json::Map<String, Value>,
) -> Value {
    let mut obj = serde_json::Map::new();

    obj.insert("info".to_string(), construct_album_info(ctx));
    obj.insert("ALBUM".to_string(), resolvers::resolve_top_level_album_key("ALBUM", ctx));
    obj.insert("ALBUMARTIST".to_string(), resolvers::resolve_top_level_album_key("ALBUMARTIST", ctx));
    obj.insert("DATE".to_string(), resolvers::resolve_top_level_album_key("DATE", ctx));
    obj.insert("GENRE".to_string(), resolvers::resolve_top_level_album_key("GENRE", ctx));
    obj.insert("COMMENT".to_string(), resolvers::resolve_top_level_album_key("COMMENT", ctx));
    obj.insert("ORIGINAL_YYYY_MM".to_string(), resolvers::resolve_top_level_album_key("ORIGINAL_YYYY_MM", ctx));
    obj.insert("RELEASE_YYYY_MM".to_string(), resolvers::resolve_top_level_album_key("RELEASE_YYYY_MM", ctx));

    let mut tags = serde_json::Map::new();
    for (key, meta) in registry {
        let key_lower = key.to_lowercase();
        if ["album", "albumartist", "date", "genre", "comment", "original_yyyy_mm", "release_yyyy_mm"].contains(&key_lower.as_str()) {
            continue;
        }
        let val = resolvers::resolve_album_key(key, meta, ctx).unwrap_or(Value::Null);
        tags.insert(key.to_uppercase(), val);
    }
    obj.insert("tags".to_string(), Value::Object(tags));
    Value::Object(obj)
}
