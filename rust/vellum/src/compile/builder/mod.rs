pub mod assets;
pub mod context;
pub mod scan;

use vellum_core::error::VellumError;
use crate::compile::builder::context::{AlbumContext, TrackContext};
use crate::compile::resolvers;
use crate::expand_path;
use crate::harvest;
use serde_json::{Value, json, Map};
use sha2::Digest;
use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;

pub fn build(
    album_root: &Path,
    project_root: &Path,
    config: &Value,
    manifest_cfg: &Value,
    _active_flags: &[String],
) -> Result<Value, VellumError> {
    let metadata_path = album_root.join("metadata.toml");
    let meta = std::fs::metadata(&metadata_path)
        .map_err(|_| VellumError::MissingPrimaryManifest { path: album_root.to_path_buf() })?;
    
    let meta_mtime = meta
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut manifests_mtime_sum: u64 = meta_mtime;
    if let Some(manifests) = config.get("compiler").and_then(|c| c.get("manifests")).and_then(Value::as_array) {
        for m_val in manifests {
            if let Some(m_name) = m_val.as_str() {
                let m_path = album_root.join(m_name);
                if m_path.exists() {
                    manifests_mtime_sum += std::fs::metadata(&m_path)
                        .and_then(|m| m.modified())
                        .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs())
                        .unwrap_or(0);
                }
            }
        }
    }

    let content = std::fs::read_to_string(&metadata_path)?;
    let meta_hash = format!("{:x}", sha2::Sha256::digest(content.as_bytes()));

    let parsed_toml = toml::from_str::<toml::Value>(&content)
        .map_err(|source| VellumError::ManifestParseError { path: metadata_path.clone(), source })?;
    let mut metadata_json = normalize_keys(serde_json::to_value(parsed_toml)?);

    if let Some(manifests) = config.get("compiler").and_then(|c| c.get("manifests")).and_then(Value::as_array) {
        for m_val in manifests {
            if let Some(m_name) = m_val.as_str() {
                let m_path = album_root.join(m_name);
                if !m_path.exists() {
                    continue;
                }
                let m_content = std::fs::read_to_string(&m_path)?;
                let parsed_aux = toml::from_str::<toml::Value>(&m_content)
                    .map_err(|source| VellumError::ManifestParseError { path: m_path.clone(), source })?;
                let m_json = normalize_keys(serde_json::to_value(parsed_aux)?);
                
                if let Some(aux_album) = m_json.get("album").and_then(Value::as_object) {
                    if let Some(primary_album) = metadata_json.get_mut("album").and_then(Value::as_object_mut) {
                        for (k, v) in aux_album {
                            if !primary_album.contains_key(k) {
                                primary_album.insert(k.clone(), v.clone());
                            }
                        }
                    }
                }

                if let Some(aux_tracks) = m_json.get("tracks").and_then(Value::as_array) {
                    if !aux_tracks.is_empty() {
                        let primary_tracks = metadata_json.get_mut("tracks")
                            .and_then(Value::as_array_mut)
                            .ok_or_else(|| VellumError::MissingTracksBlock { path: album_root.to_path_buf() })?;

                        if aux_tracks.len() != primary_tracks.len() {
                            return Err(VellumError::TrackCountMismatch {
                                manifest: m_name.to_string(),
                                path: album_root.to_path_buf(),
                                primary_count: primary_tracks.len(),
                                aux_count: aux_tracks.len(),
                            });
                        }

                        let mut seen_identities = HashSet::new();
                        for (idx, aux_t) in aux_tracks.iter().enumerate() {
                            let a_obj = aux_t.as_object().ok_or_else(|| VellumError::InvalidManifestEntry { 
                                manifest: m_name.to_string(), 
                                path: album_root.to_path_buf(), 
                                index: idx + 1 
                            })?;
                            
                            let track_no = extract_strict_u32(aux_t.get("tracknumber"), "tracknumber", None)
                                .map_err(|_| VellumError::MissingTrackIdentity {
                                    manifest: m_name.to_string(),
                                    path: album_root.to_path_buf(),
                                    index: idx + 1,
                                })?;
                            
                            let disc_no = extract_strict_u32(aux_t.get("discnumber"), "discnumber", Some(1))?;

                            if !seen_identities.insert((disc_no, track_no)) {
                                return Err(VellumError::DuplicateTrackIdentity {
                                    manifest: m_name.to_string(),
                                    path: album_root.to_path_buf(),
                                    disc: disc_no,
                                    track: track_no,
                                });
                            }

                            let mut found = false;
                            for prim_t in primary_tracks.iter_mut() {
                                let p_track_no: u32 = extract_strict_u32(prim_t.get("tracknumber"), "tracknumber", None)?;
                                let p_disc_no: u32 = extract_strict_u32(prim_t.get("discnumber"), "discnumber", Some(1))?;

                                if track_no == p_track_no && disc_no == p_disc_no {
                                    if let Some(p_obj) = prim_t.as_object_mut() {
                                        for (k, v) in a_obj {
                                            if k != "tracknumber" && k != "discnumber" {
                                                if !p_obj.contains_key(k) {
                                                    let val: Value = v.clone();
                                                    p_obj.insert(k.clone(), val);
                                                }
                                            }
                                        }
                                    }
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                return Err(VellumError::OrphanedAuxiliaryData {
                                    manifest: m_name.to_string(),
                                    path: album_root.to_path_buf(),
                                    disc: disc_no,
                                    track: track_no,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let (c_path, c_hash, c_mtime, c_size) = assets::resolve_cover_info(album_root);
    let loaded_image =
        assets::load_or_create_thumbnail(config, album_root, c_path.as_deref(), &c_hash);

    let mut cover_metrics = None;
    if !c_hash.is_empty() {
        let cache_str = config.get("storage").and_then(|s| s.get("cache")).and_then(Value::as_str).unwrap_or("~/.cache/vellum");
        let cache_root = crate::expand_path(cache_str);
        let metrics_dir = cache_root.join("cover_data");
        std::fs::create_dir_all(&metrics_dir).ok();
        
        let metrics_path = metrics_dir.join(format!("{}.json", c_hash));
        
        let palette_cfg = config.get("compiler").and_then(|c| c.get("cover_palette"));
        let cover_palette_raw = metadata_json.get("album").and_then(|a| a.get("cover_palette").or_else(|| a.get("COVER_PALETTE")));
        
        let palette_params = format!("{:?}|{:?}", palette_cfg, cover_palette_raw);
        
        let mut metrics = if metrics_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&metrics_path) {
                serde_json::from_str::<assets::CoverMetrics>(&content).ok()
            } else { None }
        } else { None }.unwrap_or_else(|| assets::CoverMetrics {
            hash: c_hash.clone(),
            entropy: None,
            chroma: None,
            palette: None,
            palette_params: None,
        });
        
        let mut needs_save = false;
        
        if let Some(ref img) = loaded_image {
            if metrics.chroma.is_none() {
                metrics.chroma = Some(assets::calculate_chroma(img));
                needs_save = true;
            }
            if metrics.entropy.is_none() {
                metrics.entropy = Some(assets::calculate_entropy(img));
                needs_save = true;
            }
            
            if metrics.palette_params.as_deref() != Some(&palette_params) || metrics.palette.is_none() {
                if let Some(palette_val) = resolvers::cover_palette::resolve_core(img, palette_cfg, cover_palette_raw) {
                    metrics.palette = Some(palette_val);
                    metrics.palette_params = Some(palette_params);
                    needs_save = true;
                }
            }
        }
        
        if needs_save {
            if let Ok(content) = serde_json::to_string(&metrics) {
                let _ = std::fs::write(&metrics_path, content);
            }
        }
        
        cover_metrics = Some(metrics);
    }

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
        .ok_or_else(|| VellumError::MissingTracksBlock { path: album_root.to_path_buf() })?;

    if audio_files.len() != track_entries.len() {
        return Err(VellumError::PhysicalInventoryMismatch {
            path: album_root.to_path_buf(),
            files_count: audio_files.len(),
            tracks_count: track_entries.len(),
        });
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
        .ok_or_else(|| VellumError::MissingCompilerRegistry)?
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
                                if let (Some(existing_obj), Some(new_obj)) = (existing.as_object_mut() as Option<&mut Map<String, Value>>, v.as_object()) {
                                    for (nk, nv) in new_obj {
                                        let val: Value = nv.clone();
                                        existing_obj.insert(nk.clone(), val);
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
        manifests_mtime_sum,
        cover_hash: &c_hash,
        cover_path: c_path.as_deref(),
        cover_mtime: c_mtime,
        cover_byte_size: c_size,
        cover_metrics: cover_metrics.as_ref(),
        config,
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

fn extract_strict_u32(val: Option<&Value>, name: &str, default: Option<u32>) -> Result<u32, VellumError> {
    let Some(v) = val else {
        if let Some(d) = default {
            return Ok(d);
        }
        return Err(VellumError::InvalidIdentityFormat {
            field: name.to_string(),
            message: "Missing expected integer".to_string(),
        });
    };
    match v {
        Value::Number(n) => n
            .as_u64()
            .and_then(|i| u32::try_from(i).ok())
            .ok_or_else(|| VellumError::InvalidIdentityFormat {
                field: name.to_string(),
                message: "Value exceeds 32-bit integer limits".to_string(),
            }),
        Value::String(s) => {
            let base = s.split('/').next().unwrap_or("").trim();
            base.parse::<u32>().map_err(|_| VellumError::InvalidIdentityFormat {
                field: name.to_string(),
                message: format!("Cannot interpret string '{}' as integer", s),
            })
        }
        Value::Null => {
            if let Some(d) = default {
                Ok(d)
            } else {
                Err(VellumError::InvalidIdentityFormat {
                    field: name.to_string(),
                    message: "Field cannot be null".to_string(),
                })
            }
        }
        _ => Err(VellumError::InvalidIdentityFormat {
            field: name.to_string(),
            message: "Unsupported data type found".to_string(),
        }),
    }
}

fn validate_album_level_keys(
    album_source: &Value,
    track_entries: &[Value],
    registry: &Map<String, Value>,
    album_root: &Path,
) -> Result<(), VellumError> {
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
                        return Err(VellumError::GlobalKeyConflict {
                            path: album_root.to_path_buf(),
                            key: key.clone(),
                            val1: first_val.to_string(),
                            source1: source_name.clone(),
                            val2: v.to_string(),
                            index: idx + 1,
                        });
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
    registry: &Map<String, Value>,
) -> Result<(Vec<Value>, Vec<Value>), VellumError> {
    let mut harvested_spine = Vec::new();
    for path in audio_files {
        harvested_spine.push(harvest::harvest_file(&path).map_err(|source| VellumError::HarvestError { path: path.clone(), source })?);
    }

    let mut total_discs = 1;
    for t in track_entries {
        if let Ok(d) = extract_strict_u32(t.get("discnumber"), "discnumber", Some(1)) {
            if d > total_discs {
                total_discs = d;
            }
        }
    }

    let mut final_tracks = Vec::new();
    let mut harvested_cache = Vec::new();

    for (idx, h_data) in harvested_spine.into_iter().enumerate() {
        let track_number: u32 = extract_strict_u32(track_entries[idx].get("tracknumber"), "tracknumber", None)
            .map_err(|_| VellumError::MissingTrackIdentity {
                manifest: "metadata.toml".to_string(),
                path: album_root.to_path_buf(),
                index: idx + 1,
            })?;
        let disc_number: u32 = extract_strict_u32(track_entries[idx].get("discnumber"), "discnumber", Some(1))?;

        let t_ctx = TrackContext {
            track_number,
            disc_number,
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

fn validate_track_indices(entries: &[Value], root: &Path) -> Result<(), VellumError> {
    let mut seen_ids = HashSet::new();
    for (idx, entry) in entries.iter().enumerate() {
        let t = extract_strict_u32(entry.get("tracknumber"), "tracknumber", None)
            .map_err(|_| VellumError::MissingTrackIdentity {
                manifest: "metadata.toml".to_string(),
                path: root.to_path_buf(),
                index: idx + 1,
            })?;
        let d = extract_strict_u32(entry.get("discnumber"), "discnumber", Some(1))?;

        if !seen_ids.insert((d, t)) {
            return Err(VellumError::DuplicateTrackIdentity {
                manifest: "metadata.toml".to_string(),
                path: root.to_path_buf(),
                disc: d,
                track: t,
            });
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

    let lyrics_path = ctx
        .source
        .get("lyrics_path")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            resolvers::native::resolve_lyrics_path(
                ctx.album_root,
                ctx.track_number,
                ctx.disc_number,
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
    registry: &Map<String, Value>,
) -> Value {
    let mut obj = serde_json::Map::new();

    obj.insert("info".to_string(), construct_track_info(ctx, total_discs));

    obj.insert("TITLE".to_string(), resolvers::resolve_top_level_track_key("TITLE", ctx));
    obj.insert("ARTIST".to_string(), resolvers::resolve_top_level_track_key("ARTIST", ctx));
    obj.insert("TRACKNUMBER".to_string(), json!(ctx.track_number));
    obj.insert("DISCNUMBER".to_string(), json!(ctx.disc_number));

    let mut tags = serde_json::Map::new();
    for (key, meta) in registry {
        let level = meta.get("level").and_then(Value::as_str).unwrap_or("");
        if level != "tracks" && level != "track" {
            continue;
        }

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
    info.insert(
        "date_added".to_string(),
        resolvers::native::resolve_album_info_date_added(ctx, "").unwrap_or_else(|| json!("")),
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
    info.insert("manifests_mtime_sum".to_string(), json!(ctx.manifests_mtime_sum));
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
    registry: &Map<String, Value>,
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
        if meta.get("level").and_then(Value::as_str) != Some("album") {
            continue;
        }

        let key_lower = key.to_lowercase();
        if["album", "albumartist", "date", "genre", "comment", "original_yyyy_mm", "release_yyyy_mm"].contains(&key_lower.as_str()) {
            continue;
        }
        let val = resolvers::resolve_album_key(key, meta, ctx).unwrap_or(Value::Null);
        tags.insert(key.to_uppercase(), val);
    }

    if let Some(palette_cfg) = ctx.config.get("compiler").and_then(|c| c.get("cover_palette")) {
        if let Some(val) = resolvers::native::resolve_cover_palette(ctx, palette_cfg) {
            tags.insert("COVER_PALETTE".to_string(), val);
        }
    }

    obj.insert("tags".to_string(), Value::Object(tags));
    Value::Object(obj)
}
