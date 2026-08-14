use crate::compile::{album, context, covers, tracks, utils};
use libdale::compiler::manifest::load_manifests;
use libdale::error::DaleError;
use serde::de::Error as _;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

struct PrimaryBuildData {
    parsed_manifests: serde_json::Map<String, Value>,
    primary_tracks: Vec<Value>,
    prep_ctx: context::PreparedContext,
}

pub struct BuildOutput {
    pub lock_json: Value,
    pub dependencies: HashSet<PathBuf>,
}

struct DispatcherInput<'a> {
    config: &'a libdale::lua::ResolvedConfig,
    album_root: &'a Path,
    music_directory: &'a Path,
    total_discs: u32,
    total_tracks: u32,
    duration_sum_ms: u64,
    cover_metrics: Option<&'a libdale::models::CoverMetrics>,
}

struct AlbumHeader<'a> {
    artist: &'a str,
    title: &'a str,
    date: &'a str,
    id: &'a str,
}

struct AlbumAssemblyInput<'a> {
    header: AlbumHeader<'a>,
    keys: Value,
    info: Value,
    manifests: &'a BTreeMap<String, Value>,
    cover_file_info: &'a Value,
    parsed_manifests: &'a serde_json::Map<String, Value>,
    album_root: &'a Path,
}

fn load_primary_and_files(
    album_root: &Path,
    config: &libdale::lua::ResolvedConfig,
    cache_root: &Path,
) -> Result<PrimaryBuildData, DaleError> {
    let manifest_names = config.app.compiler.manifests.as_deref();
    let parsed_manifests = load_manifests(album_root, manifest_names, cache_root)?;

    let primary_manifest = parsed_manifests.get("metadata").ok_or_else(|| {
        DaleError::MissingPrimaryManifest {
            path: album_root.to_path_buf(),
        }
    })?;
    let primary_tracks = primary_manifest
        .get("tracks")
        .and_then(Value::as_array)
        .ok_or_else(|| DaleError::MissingTracksBlock {
            path: album_root.to_path_buf(),
        })?
        .clone();

    let prep_ctx = context::prepare_build_context(config, album_root);
    Ok(PrimaryBuildData {
        parsed_manifests,
        primary_tracks,
        prep_ctx,
    })
}

fn build_dispatcher_context(
    input: &DispatcherInput<'_>,
    ctx_tracks: &[Value],
) -> (Value, String, String, String, String) {
    let config_json = serde_json::to_value(&input.config.app).unwrap_or_else(|_| json!({}));
    let id_str = libdale::resolvers::rel_path(input.album_root, input.music_directory);

    let project_root_str = input
        .config
        .path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy()
        .into_owned();
    let album_root_str = input.album_root.to_string_lossy().into_owned();
    let music_directory_str = input.music_directory.to_string_lossy().into_owned();

    let ctx = json!({
        "config": config_json,
        "id": id_str,
        "paths": {
            "album_root": album_root_str,
            "project_root": project_root_str,
            "music_directory": music_directory_str
        },
        "total_discs": input.total_discs,
        "total_tracks": input.total_tracks,
        "duration_milliseconds": input.duration_sum_ms,
        "cover_metrics": input.cover_metrics,
        "tracks": ctx_tracks,
    });

    (
        ctx,
        id_str,
        album_root_str,
        project_root_str,
        music_directory_str,
    )
}

fn assemble_album_object(
    input: &AlbumAssemblyInput<'_>,
) -> Result<serde_json::Map<String, Value>, DaleError> {
    let mut album_obj = serde_json::Map::new();
    album_obj.insert("albumartist".to_string(), json!(input.header.artist));
    album_obj.insert("album".to_string(), json!(input.header.title));
    album_obj.insert("date".to_string(), json!(input.header.date));
    album_obj.insert("id".to_string(), json!(input.header.id));
    album_obj.insert("keys".to_string(), input.keys.clone());
    album_obj.insert("info".to_string(), input.info.clone());
    album_obj.insert("manifests".to_string(), json!(input.manifests));

    let covers_entry = if input.cover_file_info.is_null() {
        Value::Null
    } else {
        json!({ "main": { "file": input.cover_file_info } })
    };
    album_obj.insert("covers".to_string(), covers_entry);

    if let Some(theme) = input.parsed_manifests.get("theme")
        && let Some(colors) = theme.get("album").and_then(|a| a.get("colors"))
    {
        let colors_validated = utils::validate_and_format_colors(colors, input.album_root)?;
        album_obj.insert("colors".to_string(), colors_validated);
    }

    Ok(album_obj)
}

fn run_dispatcher_phase(
    engine: &libdale::lua::LuaEngine,
    album_root: &Path,
    parsed_manifests: &serde_json::Map<String, Value>,
    ctx: &Value,
) -> Result<(Value, HashSet<PathBuf>), DaleError> {
    let manifests_json = Value::Object(parsed_manifests.clone());
    let lua_res = engine
        .execute_dispatcher(ctx, &manifests_json)
        .map_err(|e| DaleError::ManifestParseError {
            path: album_root.to_path_buf(),
            source: toml::de::Error::custom(e.to_string()),
        })?;

    let mut dependencies = engine
        .lua
        .app_data_ref::<libdale::lua::EngineContext>()
        .map_or_else(HashSet::new, |c| c.take_dependencies());

    for name in parsed_manifests.keys() {
        let manifest_path = album_root.join(format!("{name}.toml"));
        let canon = manifest_path.canonicalize().unwrap_or(manifest_path);
        dependencies.insert(canon);
    }

    Ok((lua_res, dependencies))
}

pub fn build(
    album_root: &Path,
    config: &libdale::lua::ResolvedConfig,
    engine: &libdale::lua::LuaEngine,
) -> Result<BuildOutput, DaleError> {
    if let Some(ctx) = engine.lua.app_data_ref::<libdale::lua::EngineContext>() {
        ctx.take_dependencies();
    }

    let cache_root = libdale::utils::expand_path(&config.app.storage.cache);
    let build_data = load_primary_and_files(album_root, config, &cache_root)?;

    let lib_hash = crate::update::cache::calculate_hash(
        &build_data.prep_ctx.music_directory.to_string_lossy(),
    );
    let (cover_file_info, cover_metrics) =
        covers::resolve_cover_data_cached(album_root, config, &lib_hash);

    let is_virtual = album::is_virtual_album(&build_data.parsed_manifests);
    tracks::validate_audio_files(
        is_virtual,
        &build_data.prep_ctx.audio_files,
        &build_data.primary_tracks,
        album_root,
    )?;

    let lock_manifests =
        album::generate_lock_manifests(&build_data.parsed_manifests, album_root, is_virtual);
    let total_discs = libdale::resolvers::calculate_total_discs(&build_data.primary_tracks);
    let total_tracks = build_data.primary_tracks.len() as u32;

    let (ctx_tracks, duration_sum_ms) = tracks::build_ctx_tracks(
        is_virtual,
        &build_data.primary_tracks,
        &build_data.prep_ctx.audio_files,
        album_root,
        &cache_root,
    )?;

    let disp_input = DispatcherInput {
        config,
        album_root,
        music_directory: &build_data.prep_ctx.music_directory,
        total_discs,
        total_tracks,
        duration_sum_ms,
        cover_metrics: cover_metrics.as_ref(),
    };

    let (ctx, id_str, album_root_str, project_root_str, music_directory_str) =
        build_dispatcher_context(&disp_input, &ctx_tracks);

    let (lua_res, dependencies) =
        run_dispatcher_phase(engine, album_root, &build_data.parsed_manifests, &ctx)?;

    let mut album_keys = lua_res.get("album").cloned().unwrap_or_else(|| json!({}));
    utils::sort_json_keys(&mut album_keys);

    let track_keys_array = lua_res.get("tracks").and_then(Value::as_array);

    let empty_album = json!({});
    let primary_manifest = build_data.parsed_manifests.get("metadata").ok_or_else(|| {
        DaleError::MissingPrimaryManifest {
            path: album_root.to_path_buf(),
        }
    })?;
    let primary_album = primary_manifest.get("album").unwrap_or(&empty_album);
    let (albumartist, album_title, date) =
        album::parse_mandatory_album_fields(primary_album, album_root)?;

    let info_obj = json!({
        "virtual": is_virtual,
        "total_discs": total_discs,
        "total_tracks": total_tracks,
        "duration_milliseconds": duration_sum_ms,
        "duration_formatted": libdale::resolvers::format_ms(duration_sum_ms),
    });

    let final_tracks = tracks::build_final_tracks(
        &build_data.primary_tracks,
        &albumartist,
        track_keys_array,
        &ctx_tracks,
        album_root,
    )?;

    let assembly_input = AlbumAssemblyInput {
        header: AlbumHeader {
            artist: &albumartist,
            title: &album_title,
            date: &date,
            id: &id_str,
        },
        keys: album_keys,
        info: info_obj,
        manifests: &lock_manifests,
        cover_file_info: &cover_file_info,
        parsed_manifests: &build_data.parsed_manifests,
        album_root,
    };

    let album_obj = assemble_album_object(&assembly_input)?;

    let mut final_json = serde_json::Map::new();
    final_json.insert("album".to_string(), Value::Object(album_obj));
    final_json.insert("tracks".to_string(), Value::Array(final_tracks));
    final_json.insert(
        "ctx".to_string(),
        json!({
            "paths": {
                "album_root": album_root_str,
                "project_root": project_root_str,
                "music_directory": music_directory_str
            }
        }),
    );

    Ok(BuildOutput {
        lock_json: Value::Object(final_json),
        dependencies,
    })
}
