pub mod cache;
pub mod verify;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cache::{
    calculate_hash, get_lua_config_hash, load_cache, save_cache, validate_library_root, FileStat,
};
use crate::compile;
use libdale::utils::expand_path;
use verify::{find_missing_paths, verify_albums_parallel};

pub async fn run(
    target_path: Option<PathBuf>,
    force: bool,
    jobs: Option<usize>,
    silent: bool,
) -> Result<()> {
    if is_server_up(8000).await {
        return run_delegated_update(target_path, force, jobs, silent, 8000).await;
    }

    let config = libdale::lua::ResolvedConfig::load().context("Failed to load config")?;
    let music_directory = expand_path(&config.app.storage.music_directory)
        .canonicalize()
        .context("Invalid music_directory")?;

    let effective_jobs = jobs.or(config.app.compiler.jobs);

    let lib_hash = calculate_hash(&music_directory.to_string_lossy());
    let cache_root = expand_path(&config.app.storage.cache);
    let base_cache_dir = cache_root.join("libraries").join(&lib_hash);
    fs::create_dir_all(&base_cache_dir)?;

    validate_library_root(&base_cache_dir, &lib_hash).await?;

    let mut cache = load_cache(&base_cache_dir.join("library.json"));

    let (config_changed, lua_hash_file, lua_hash) =
        check_lua_config_changed(&config.dependencies, &cache_root, silent);
    let force = force || config_changed;

    let scan_root = target_path.map_or_else(
        || music_directory.clone(),
        |p| p.canonicalize().unwrap_or(p),
    );

    let (work_queue, missing_paths) = resolve_work_queue(
        &scan_root,
        &music_directory,
        &cache,
        force,
        effective_jobs,
        None,
        silent,
    )?;

    if work_queue.is_empty() && missing_paths.is_empty() {
        if !silent {
            log::info!("Library is up to date.");
        }
        let _ = fs::write(&lua_hash_file, &lua_hash);
        save_cache(&cache, &base_cache_dir.join("library.json"))?;
        return Ok(());
    }

    let checked_count = work_queue.len();
    let missing_count = missing_paths.len();
    let start_time = std::time::Instant::now();

    let compile_options = compile::CompileOptions {
        target_path: scan_root,
        flags: vec!["default".to_string()],
        specific_albums: Some(work_queue.clone()),
        jobs: effective_jobs,
        compile_flags: compile::CompileFlags {
            mode: compile::CompileMode::Standard,
            target: compile::ExportTarget::File,
            pretty: false,
        },
        ingest_tx: None,
        active_writes: None,
    };

    let written_count = compile::run(compile_options).await?;

    update_cache_entries(&mut cache, &work_queue, &missing_paths, &music_directory, silent);

    let elapsed = start_time.elapsed().as_millis();
    if !silent {
        log::info!(
            "Update complete: {checked_count} checked ({written_count} modified), {missing_count} removed. Finished in {elapsed}ms."
        );
    }

    let _ = fs::write(&lua_hash_file, &lua_hash);
    save_cache(&cache, &base_cache_dir.join("library.json"))?;

    Ok(())
}

async fn run_delegated_update(
    target_path: Option<PathBuf>,
    force: bool,
    jobs: Option<usize>,
    silent: bool,
    port: u16,
) -> Result<()> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "path": target_path.map(|p| p.to_string_lossy().to_string()),
        "force": force,
        "jobs": jobs,
        "silent": silent,
    });

    let mut res = client
        .post(format!("http://127.0.0.1:{port}/api/internal/trigger_update"))
        .json(&body)
        .send()
        .await
        .context("Failed to connect to running server for delegated update")?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        anyhow::bail!("Server update failed: {text}");
    }

    let mut buffer = String::new();

    while let Ok(Some(chunk)) = res.chunk().await {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim_end().to_string();
            buffer.drain(..=pos);
            if !line.is_empty() {
                log::info!("{line}");
            }
        }
    }

    if !buffer.trim().is_empty() {
        let line = buffer.trim();
        log::info!("{line}");
    }

    Ok(())
}

fn emit_log(msg: String, silent: bool, log_tx: Option<&tokio::sync::mpsc::Sender<String>>) {
    if let Some(tx) = log_tx {
        let _ = tx.try_send(msg);
    } else if !silent {
        log::info!("{msg}");
    }
}

fn spawn_server_ingest_handler(
    state: Arc<crate::server::state::AppState>,
    silent: bool,
    log_tx: Option<tokio::sync::mpsc::Sender<String>>,
) -> (
    tokio::sync::mpsc::Sender<crate::server::api::system::AlbumIngestPayload>,
    tokio::task::JoinHandle<()>,
) {
    let (ingest_tx, mut ingest_rx) =
        tokio::sync::mpsc::channel::<crate::server::api::system::AlbumIngestPayload>(512);

    let ingest_handle = tokio::spawn(async move {
        while let Some(payload) = ingest_rx.recv().await {
            if !payload.lock_json.is_empty() && (!payload.artist.is_empty() || !payload.album.is_empty()) {
                let artist = &payload.artist;
                let album = &payload.album;
                emit_log(
                    format!("Updated: {artist} - {album}"),
                    silent,
                    log_tx.as_ref(),
                );
            }
            let item = vec![(payload.id, payload.lock_json, payload.eval_res)];
            crate::server::inotify::handler::ingest_and_broadcast_albums(item, &state).await;
        }
    });

    (ingest_tx, ingest_handle)
}

pub async fn run_server_update(
    state: Arc<crate::server::state::AppState>,
    target_path: Option<PathBuf>,
    force: bool,
    jobs: Option<usize>,
    silent: bool,
    log_tx: Option<tokio::sync::mpsc::Sender<String>>,
) -> Result<()> {
    let config = state.config.read().await.app.clone();
    let music_directory = state.config.read().await.music_directory.clone();
    let cache_root = state.config.read().await.cache_root.clone();
    let dependencies = state.config.read().await.resolved_dependencies.clone();

    let effective_jobs = jobs.or(config.compiler.jobs);
    let is_full_library =
        target_path.is_none() || target_path.as_deref() == Some(music_directory.as_path());

    let lib_hash = calculate_hash(&music_directory.to_string_lossy());
    let base_cache_dir = cache_root.join("libraries").join(&lib_hash);
    fs::create_dir_all(&base_cache_dir)?;

    validate_library_root(&base_cache_dir, &lib_hash).await?;

    let mut cache = load_cache(&base_cache_dir.join("library.json"));

    let (config_changed, lua_hash_file, lua_hash) =
        check_lua_config_changed(&dependencies, &cache_root, silent);
    let force = force || config_changed;

    let scan_root = target_path.map_or_else(
        || music_directory.clone(),
        |p| p.canonicalize().unwrap_or(p),
    );

    let tracked = if !force && is_full_library {
        try_get_server_tracked_albums(&music_directory).await
    } else {
        None
    };

    let (work_queue, missing_paths) = resolve_work_queue(
        &scan_root,
        &music_directory,
        &cache,
        force,
        effective_jobs,
        tracked,
        silent,
    )?;

    if work_queue.is_empty() && missing_paths.is_empty() {
        emit_log("Library is up to date.".to_string(), silent, log_tx.as_ref());
        let _ = fs::write(&lua_hash_file, &lua_hash);
        save_cache(&cache, &base_cache_dir.join("library.json"))?;
        return Ok(());
    }

    let checked_count = work_queue.len();
    let missing_count = missing_paths.len();
    let start_time = std::time::Instant::now();

    let (ingest_tx, ingest_handle) =
        spawn_server_ingest_handler(Arc::clone(&state), silent, log_tx.clone());

    for missing in &missing_paths {
        let album_id = libdale::resolvers::rel_path(missing, &music_directory);
        let display_path = missing.strip_prefix(&music_directory).unwrap_or(missing);
        emit_log(
            format!("Removed: {}", display_path.display()),
            silent,
            log_tx.as_ref(),
        );
        let _ = ingest_tx
            .send(crate::server::api::system::AlbumIngestPayload {
                id: album_id,
                artist: String::new(),
                album: String::new(),
                lock_json: String::new(),
                eval_res: None,
            })
            .await;
    }

    emit_log(format!("Verifying {checked_count} albums..."), silent, log_tx.as_ref());

    let compile_options = compile::CompileOptions {
        target_path: scan_root,
        flags: vec!["default".to_string()],
        specific_albums: Some(work_queue.clone()),
        jobs: effective_jobs,
        compile_flags: compile::CompileFlags {
            mode: compile::CompileMode::Standard,
            target: compile::ExportTarget::File,
            pretty: false,
        },
        ingest_tx: Some(ingest_tx),
        active_writes: Some(Arc::clone(&state.active_writes)),
    };

    let written_count = compile::run(compile_options).await?;

    let _ = ingest_handle.await;

    update_cache_entries(&mut cache, &work_queue, &missing_paths, &music_directory, silent);

    let elapsed = start_time.elapsed().as_millis();
    emit_log(
        format!("Update complete: {checked_count} checked ({written_count} modified), {missing_count} removed. Finished in {elapsed}ms."),
        silent,
        log_tx.as_ref(),
    );

    let _ = fs::write(&lua_hash_file, &lua_hash);
    save_cache(&cache, &base_cache_dir.join("library.json"))?;

    Ok(())
}

fn resolve_work_queue(
    scan_root: &Path,
    music_directory: &Path,
    cache: &std::collections::HashMap<String, FileStat>,
    force: bool,
    effective_jobs: Option<usize>,
    tracked: Option<(Vec<PathBuf>, Vec<PathBuf>)>,
    silent: bool,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    if !force && let Some((wq, mp)) = tracked {
        if !silent && !wq.is_empty() {
            log::info!("Verifying {} tracked albums...", wq.len());
        }

        let results = verify_albums_parallel(wq, cache, force, effective_jobs, music_directory)?;
        let mut verified_wq = Vec::new();
        for (path, is_dirty) in results {
            if is_dirty {
                verified_wq.push(path);
            }
        }
        Ok((verified_wq, mp))
    } else {
        let all_albums = libdale::scanner::find_target_albums(scan_root)?;
        let mp = find_missing_paths(&all_albums, music_directory, scan_root, cache);

        if !silent {
            log::info!("Verifying {} albums...", all_albums.len());
        }

        let results = verify_albums_parallel(all_albums, cache, force, effective_jobs, music_directory)?;
        let mut wq = Vec::new();
        for (path, is_dirty) in results {
            if is_dirty {
                wq.push(path);
            }
        }
        Ok((wq, mp))
    }
}

async fn is_server_up(port: u16) -> bool {
    let client = reqwest::Client::new();
    client
        .get(format!("http://127.0.0.1:{port}/api/internal/tracked_albums"))
        .timeout(std::time::Duration::from_millis(300))
        .send()
        .await
        .is_ok_and(|r| r.status().is_success())
}

fn update_cache_entries(
    cache: &mut std::collections::HashMap<String, FileStat>,
    work_queue: &[PathBuf],
    missing_paths: &[PathBuf],
    music_directory: &Path,
    silent: bool,
) {
    for album_root in work_queue {
        let album_id = libdale::resolvers::rel_path(album_root, music_directory);
        let prefix = if album_id.is_empty() {
            String::new()
        } else {
            format!("{album_id}/")
        };

        cache.retain(|k, _| {
            if prefix.is_empty() {
                k.contains('/')
            } else {
                !k.starts_with(&prefix)
            }
        });

        for entry in walkdir::WalkDir::new(album_root)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            let p = entry.path();
            if p.is_file() {
                let rel = libdale::resolvers::rel_path(p, music_directory);
                if let Ok(m) = entry.metadata() {
                    let mtime = m
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let size = m.len();
                    cache.insert(rel, FileStat { mtime, size });
                }
            }
        }
    }

    for missing in missing_paths {
        let album_id = libdale::resolvers::rel_path(missing, music_directory);
        let prefix = format!("{album_id}/");

        if !silent {
            let display_path = missing.strip_prefix(music_directory).unwrap_or(missing);
            log::info!("Removed: {}", display_path.display());
        }

        cache.retain(|k, _| !k.starts_with(&prefix));
    }
}

async fn try_get_server_tracked_albums(
    music_directory: &Path,
) -> Option<(Vec<PathBuf>, Vec<PathBuf>)> {
    let resp = reqwest::Client::new()
        .get("http://127.0.0.1:8000/api/internal/tracked_albums")
        .timeout(std::time::Duration::from_millis(500))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let data: serde_json::Value = resp.json().await.ok()?;
    if data
        .get("full_rescan")
        .and_then(serde_json::Value::as_bool)
        != Some(false)
    {
        return None;
    }

    let tracked_arr = data.get("tracked_albums")?.as_array()?;
    let mut work_queue = Vec::new();
    let mut missing_paths = Vec::new();

    for item in tracked_arr {
        if let Some(id) = item.as_str() {
            let p = music_directory.join(id);
            if p.exists() {
                work_queue.push(p);
            } else {
                missing_paths.push(p);
            }
        }
    }

    Some((work_queue, missing_paths))
}

fn check_lua_config_changed(
    dependencies: &[PathBuf],
    cache_root: &Path,
    silent: bool,
) -> (bool, PathBuf, String) {
    let lua_hash = get_lua_config_hash(dependencies);
    let lua_hash_file = cache_root.join("config.blake3");
    let previous_lua_hash = fs::read_to_string(&lua_hash_file).unwrap_or_default();
    let config_changed = lua_hash != previous_lua_hash;

    if config_changed && !silent {
        log::info!("Lua configuration changed. Re-evaluating album locks...");
    }

    (config_changed, lua_hash_file, lua_hash)
}
