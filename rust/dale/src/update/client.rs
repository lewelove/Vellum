use crate::compile;
use crate::server::state::{DependencyGraph, UpdateScope};
use crate::update::cache::{
    calculate_hash, load_cache, save_cache, validate_library_root,
};
use crate::update::queue::{
    check_lua_config_changed, resolve_work_queue, update_cache_entries,
};
use anyhow::{Context, Result};
use libdale::utils::expand_path;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

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
    let mut deps_graph = DependencyGraph::load_from_file(&base_cache_dir.join("dependencies.json"));

    let (config_changed, lua_hash_file, lua_hash) =
        check_lua_config_changed(&config.dependencies, &cache_root, silent);
    let force = force || config_changed;

    let (scan_root, scope) = resolve_target_scope(target_path.as_ref(), &music_directory);

    let (work_queue, missing_paths) = resolve_work_queue(
        &scope,
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
        let _ = deps_graph.save_to_file(&base_cache_dir.join("dependencies.json"));
        return Ok(());
    }

    let checked_count = work_queue.len();
    let missing_count = missing_paths.len();
    let start_time = std::time::Instant::now();

    let written_count = execute_compile_pass(
        scan_root,
        &work_queue,
        &missing_paths,
        &mut deps_graph,
        effective_jobs,
        silent,
    )
    .await?;

    update_cache_entries(&mut cache, &work_queue, &missing_paths, &music_directory, silent);

    let elapsed = start_time.elapsed().as_millis();
    if !silent {
        log::info!(
            "Update complete: {checked_count} checked ({written_count} modified), {missing_count} removed. Finished in {elapsed}ms."
        );
    }

    let _ = fs::write(&lua_hash_file, &lua_hash);
    save_cache(&cache, &base_cache_dir.join("library.json"))?;
    let _ = deps_graph.save_to_file(&base_cache_dir.join("dependencies.json"));

    Ok(())
}

async fn execute_compile_pass(
    scan_root: PathBuf,
    work_queue: &[PathBuf],
    missing_paths: &[PathBuf],
    deps_graph: &mut DependencyGraph,
    jobs: Option<usize>,
    silent: bool,
) -> Result<usize> {
    let (ingest_tx, mut ingest_rx) =
        tokio::sync::mpsc::channel::<crate::server::api::system::AlbumIngestPayload>(512);

    let ingest_handle = tokio::spawn(async move {
        let mut items = Vec::new();
        while let Some(payload) = ingest_rx.recv().await {
            items.push(payload);
        }
        items
    });

    let compile_options = compile::CompileOptions {
        target_path: scan_root,
        flags: vec!["default".to_string()],
        specific_albums: Some(work_queue.to_vec()),
        jobs,
        compile_flags: compile::CompileFlags {
            mode: compile::CompileMode::Standard,
            target: compile::ExportTarget::File,
            pretty: false,
        },
        ingest_tx: Some(ingest_tx),
        active_writes: None,
        silent,
    };

    let written_count = compile::run(compile_options).await?;
    let ingested_payloads = ingest_handle.await.unwrap_or_default();

    apply_ingested_deps(ingested_payloads, missing_paths, deps_graph);

    Ok(written_count)
}

fn apply_ingested_deps(
    payloads: Vec<crate::server::api::system::AlbumIngestPayload>,
    missing_paths: &[PathBuf],
    deps_graph: &mut DependencyGraph,
) {
    for payload in payloads {
        let album_path_canon = payload.album_dir.canonicalize().unwrap_or(payload.album_dir);
        if payload.eval_res.is_some() {
            deps_graph.update_album_deps(
                album_path_canon,
                payload.dependencies.into_iter().collect(),
            );
        } else {
            deps_graph.remove_album(&album_path_canon);
        }
    }

    for missing in missing_paths {
        let album_path_canon = missing.canonicalize().unwrap_or_else(|_| missing.clone());
        deps_graph.remove_album(&album_path_canon);
    }
}

fn resolve_target_scope(
    target_path: Option<&PathBuf>,
    music_directory: &Path,
) -> (PathBuf, UpdateScope) {
    let scan_root = target_path.map_or_else(
        || music_directory.to_path_buf(),
        |p| p.canonicalize().unwrap_or_else(|_| p.clone()),
    );

    let scope = target_path.map_or_else(
        || UpdateScope::All,
        |p| {
            let mut set = HashSet::new();
            set.insert(p.canonicalize().unwrap_or_else(|_| p.clone()));
            UpdateScope::Paths(set)
        },
    );

    (scan_root, scope)
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
            if !line.is_empty() && !silent {
                log::info!("{line}");
            }
        }
    }

    if !buffer.trim().is_empty() && !silent {
        let line = buffer.trim();
        log::info!("{line}");
    }

    Ok(())
}

pub async fn is_server_up(port: u16) -> bool {
    let client = reqwest::Client::new();
    client
        .get(format!("http://127.0.0.1:{port}/api/internal/tracked_albums"))
        .timeout(std::time::Duration::from_millis(300))
        .send()
        .await
        .is_ok_and(|r| r.status().is_success())
}
