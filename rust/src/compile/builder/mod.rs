pub mod assets;
pub mod scan;
pub mod context;

use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use std::path::Path;
use sha2::Digest;
use crate::compile::builder::context::{AlbumContext, TrackContext};
use crate::compile::resolvers;
use crate::harvest;
use crate::expand_path;

pub fn build(
    album_root: &Path,
    project_root: &Path,
    config: &Value,
    gen_cfg: &Value,
    _active_flags: &[String],
    no_extensions: bool,
) -> Result<(Value, bool)> {
    let metadata_path = album_root.join("metadata.toml");
    let meta = std::fs::metadata(&metadata_path)?;
    let meta_mtime = meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let meta_hash = format!("{:x}", sha2::Sha256::digest(std::fs::read(&metadata_path)?));

    let content = std::fs::read_to_string(&metadata_path)?;
    let metadata_json = normalize_keys(serde_json::to_value(toml::from_str::<toml::Value>(&content)?)?);

    let (c_path, c_hash, c_mtime, c_size) = assets::resolve_cover_info(album_root);
    let loaded_image = assets::load_or_create_thumbnail(config, album_root, c_path.as_deref(), &c_hash);

    let exts: Vec<&str> = gen_cfg.get("supported_extensions")
        .and_then(Value::as_array)
        .map_or_else(|| vec![".flac"], |arr| arr.iter().filter_map(Value::as_str).collect());
    
    let audio_files = scan::scan_audio_files(album_root, &exts);
    let track_entries = metadata_json.get("tracks").and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Missing tracks in metadata.toml"))?;

    let lib_root_raw = config.get("storage").and_then(|s| s.get("library_root")).and_then(Value::as_str).unwrap_or(".");
    let library_root = expand_path(lib_root_raw).canonicalize().unwrap_or_else(|_| expand_path(lib_root_raw));

    let registry = config.get("compiler_registry").and_then(Value::as_object).ok_or_else(|| anyhow!("Missing registry"))?;
    
    let default_source = json!({});
    let album_source = metadata_json.get("album").unwrap_or(&default_source);

    let mut requires_ext = false;
    let mut final_tracks = Vec::new();
    let mut harvested_cache = Vec::new();
    let mut o_disc = 0;
    let mut o_track = 0;
    let mut last_p_disc = None;

    let mut harvested_spine = Vec::new();
    for path in audio_files { harvested_spine.push(harvest::harvest_file(&path)?); }
    harvested_spine.sort_by(sort_harvest);

    for (idx, h_data) in harvested_spine.into_iter().enumerate() {
        let p_disc = h_data.tags.get("DISCNUMBER").and_then(|s| s.split('/').next()).and_then(|s| s.parse().ok()).unwrap_or(0);
        if Some(p_disc) == last_p_disc { o_track += 1; } else { last_p_disc = Some(p_disc); o_disc += 1; o_track = 1; }

        let t_ctx = TrackContext {
            ordinal_track_number: o_track,
            ordinal_disc_number: o_disc,
            harvest: &h_data,
            source: &track_entries[idx],
            album_source,
            album_root,
            library_root: &library_root,
        };

        let (t_obj, t_ext) = build_track(&t_ctx, registry, no_extensions);
        if t_ext { requires_ext = true; }
        final_tracks.push(t_obj);
        harvested_cache.push(serde_json::to_value(h_data)?);
    }

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

    let (album_obj, a_ext) = build_album(&album_ctx, registry, requires_ext, no_extensions);

    let final_json = json!({
        "album": album_obj,
        "tracks": final_tracks,
        "ctx": {
            "config": config,
            "metadata": metadata_json,
            "harvest": harvested_cache,
            "paths": {
                "album_root": album_root.to_string_lossy(),
                "project_root": project_root.to_string_lossy(),
                "library_root": library_root.to_string_lossy()
            }
        }
    });

    Ok((final_json, a_ext))
}

fn normalize_keys(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, val) in map { new_map.insert(k.to_lowercase(), normalize_keys(val)); }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize_keys).collect()),
        _ => v,
    }
}

fn sort_harvest(a: &harvest::TrackJson, b: &harvest::TrackJson) -> std::cmp::Ordering {
    let get_num = |t: &harvest::TrackJson, k: &str| t.tags.get(k).and_then(|s| s.split('/').next()).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let ad = get_num(a, "DISCNUMBER");
    let bd = get_num(b, "DISCNUMBER");
    if ad != bd { return ad.cmp(&bd); }
    let at = get_num(a, "TRACKNUMBER");
    let bt = get_num(b, "TRACKNUMBER");
    if at != bt { return at.cmp(&bt); }
    a.path.cmp(&b.path)
}

fn build_track(ctx: &TrackContext, registry: &serde_json::Map<String, Value>, no_ext: bool) -> (Value, bool) {
    let mut obj = serde_json::Map::new();
    let mut info = serde_json::Map::new();
    
    info.insert("track_path".to_string(), json!(resolvers::native::rel_path(&ctx.harvest.path, ctx.album_root)));
    info.insert("track_library_path".to_string(), json!(resolvers::native::rel_path(&ctx.harvest.path, ctx.library_root)));
    info.insert("track_duration".to_string(), json!(ctx.harvest.physics.duration_ms));
    info.insert("track_duration_time".to_string(), json!(resolvers::standard::format_ms(ctx.harvest.physics.duration_ms)));
    info.insert("encoding".to_string(), json!(ctx.harvest.physics.format));
    info.insert("sample_rate".to_string(), json!(ctx.harvest.physics.sample_rate));
    info.insert("bits_per_sample".to_string(), json!(ctx.harvest.physics.bit_depth.unwrap_or(0)));
    info.insert("channels".to_string(), json!(ctx.harvest.physics.channels));
    info.insert("track_mtime".to_string(), json!(ctx.harvest.physics.mtime));
    info.insert("track_byte_size".to_string(), json!(ctx.harvest.physics.file_size));

    obj.insert("info".to_string(), Value::Object(info));
    obj.insert("TITLE".to_string(), ctx.source.get("title").cloned().unwrap_or_else(|| resolvers::resolve_track_key("title", ctx).unwrap_or(Value::Null)));
    obj.insert("ARTIST".to_string(), ctx.source.get("artist").cloned().unwrap_or_else(|| resolvers::resolve_track_key("artist", ctx).unwrap_or(Value::Null)));
    obj.insert("TRACKNUMBER".to_string(), resolvers::resolve_track_key("tracknumber", ctx).unwrap_or_else(|| json!(0)));
    obj.insert("DISCNUMBER".to_string(), resolvers::resolve_track_key("discnumber", ctx).unwrap_or_else(|| json!(1)));
    
    let mut tags = serde_json::Map::new();
    let mut ext = false;
    for (key, meta) in registry {
        if meta.get("level").and_then(Value::as_str) != Some("tracks") { continue; }
        let val = ctx.source.get(key).cloned().unwrap_or_else(|| {
            if !no_ext && meta.get("provider").and_then(Value::as_str) == Some("extension") { ext = true; json!(null) }
            else { resolvers::resolve_track_key(key, ctx).unwrap_or(Value::Null) }
        });
        tags.insert(key.to_uppercase(), val);
    }
    obj.insert("tags".to_string(), Value::Object(tags));
    (Value::Object(obj), ext)
}

fn build_album(ctx: &AlbumContext, registry: &serde_json::Map<String, Value>, ext_flag: bool, no_ext: bool) -> (Value, bool) {
    let mut obj = serde_json::Map::new();
    let mut info = serde_json::Map::new();
    let dur: u64 = ctx.tracks.iter().filter_map(|t| t.get("info").and_then(|i| i.get("track_duration")).and_then(Value::as_u64)).sum();
    
    info.insert("album_path".to_string(), json!(resolvers::native::rel_path(ctx.album_root, ctx.library_root)));
    info.insert("unix_added".to_string(), json!(resolvers::native::resolve_album_info_unix_added(ctx)));
    info.insert("album_duration".to_string(), json!(dur));
    info.insert("album_duration_time".to_string(), json!(resolvers::standard::format_ms(dur)));
    info.insert("total_discs".to_string(), json!(resolvers::native::calculate_total_discs(ctx.tracks)));
    info.insert("total_tracks".to_string(), json!(ctx.tracks.len()));
    info.insert("metadata_toml_hash".to_string(), json!(ctx.meta_hash));
    info.insert("metadata_toml_mtime".to_string(), json!(ctx.meta_mtime));
    info.insert("cover_path".to_string(), json!(ctx.cover_path.unwrap_or("default_cover.png")));
    info.insert("cover_hash".to_string(), json!(ctx.cover_hash));
    info.insert("cover_mtime".to_string(), json!(ctx.cover_mtime));
    info.insert("cover_byte_size".to_string(), json!(ctx.cover_byte_size));

    obj.insert("info".to_string(), Value::Object(info));
    obj.insert("ALBUM".to_string(), ctx.source.get("album").cloned().unwrap_or_else(|| resolvers::resolve_album_key("album", ctx).unwrap_or(Value::Null)));
    obj.insert("ALBUMARTIST".to_string(), ctx.source.get("albumartist").cloned().unwrap_or_else(|| resolvers::resolve_album_key("albumartist", ctx).unwrap_or(Value::Null)));
    obj.insert("DATE".to_string(), ctx.source.get("date").cloned().unwrap_or_else(|| resolvers::resolve_album_key("date", ctx).unwrap_or(Value::Null)));
    obj.insert("GENRE".to_string(), ctx.source.get("genre").cloned().unwrap_or_else(|| resolvers::resolve_album_key("genre", ctx).unwrap_or(Value::Null)));
    obj.insert("COMMENT".to_string(), ctx.source.get("comment").cloned().unwrap_or_else(|| resolvers::resolve_album_key("comment", ctx).unwrap_or(Value::Null)));
    obj.insert("ORIGINAL_YYYY_MM".to_string(), ctx.source.get("original_yyyy_mm").cloned().unwrap_or_else(|| resolvers::resolve_album_key("original_yyyy_mm", ctx).unwrap_or(Value::Null)));
    obj.insert("RELEASE_YYYY_MM".to_string(), ctx.source.get("release_yyyy_mm").cloned().unwrap_or_else(|| resolvers::resolve_album_key("release_yyyy_mm", ctx).unwrap_or(Value::Null)));
    
    let mut tags = serde_json::Map::new();
    let mut ext = ext_flag;
    for (key, meta) in registry {
        if meta.get("level").and_then(Value::as_str) != Some("album") { continue; }
        let val = ctx.source.get(key).cloned().unwrap_or_else(|| {
            if !no_ext && meta.get("provider").and_then(Value::as_str) == Some("extension") { ext = true; json!(null) }
            else { resolvers::resolve_album_key(key, ctx).unwrap_or(Value::Null) }
        });
        tags.insert(key.to_uppercase(), val);
    }
    obj.insert("tags".to_string(), Value::Object(tags));
    (Value::Object(obj), ext)
}
