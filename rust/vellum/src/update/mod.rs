pub mod cache;
pub mod notify;
pub mod verify;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use cache::{
    calculate_hash, get_lua_config_hash, load_cache, save_cache,
    validate_library_root,
};
use crate::compile;
use libvellum::utils::expand_path;
use notify::{NotificationTaskArgs, start_notification_task};
use verify::{find_missing_paths, verify_albums_parallel};

pub async fn run(
    target_path: Option<PathBuf>,
    force: bool,
    jobs: Option<usize>,
    silent: bool,
) -> Result<()> {
    let config = libvellum::lua::ResolvedConfig::load().context("Failed to load config")?;
    let library_root = expand_path(&config.app.storage.library)
        .canonicalize()
        .context("Invalid library_root")?;

    let is_full_library = target_path.is_none() || target_path.as_deref() == Some(library_root.as_path());
    notify_if_force_update(force, is_full_library).await;

    let lib_hash = calculate_hash(&library_root.to_string_lossy());
    let cache_root = expand_path(&config.app.storage.cache);
    let base_cache_dir = cache_root.join("libraries").join(&lib_hash);
    fs::create_dir_all(&base_cache_dir)?;

    validate_library_root(&base_cache_dir, &lib_hash).await?;

    let cache_file = base_cache_dir.join("library.json");
    let cache = load_cache(&cache_file);

    let (config_changed, lua_hash_file, lua_hash) =
        check_lua_config_changed(&config.dependencies, &cache_root, silent);
    let force = force || config_changed;

    let scan_root = target_path.unwrap_or_else(|| library_root.clone());

    let all_albums = libvellum::scanner::find_target_albums(&scan_root)?;
    let missing_paths = find_missing_paths(&all_albums, &library_root, &cache);

    if !silent {
        log::info!("Verifying {} albums...", all_albums.len());
    }

    let results = verify_albums_parallel(all_albums, &cache, force, jobs, &library_root)?;
    let mut work_queue = Vec::new();

    for (path, is_dirty) in results {
        if is_dirty {
            work_queue.push(path);
        }
    }

    if work_queue.is_empty() && missing_paths.is_empty() {
        if !silent {
            log::info!("Library is up to date.");
        }
        let _ = fs::write(&lua_hash_file, &lua_hash);
        save_cache(&cache, &cache_file)?;
        return Ok(());
    }

    let dirty_count = work_queue.len();
    let missing_count = missing_paths.len();
    let start_time = std::time::Instant::now();

    let (notify_tx, notify_rx) = mpsc::channel::<compile::stream::AlbumUpdateSignal>(100);
    let cache_arc = Arc::new(Mutex::new(cache));

    let task_args = NotificationTaskArgs {
        notify_rx,
        cache_for_task: Arc::clone(&cache_arc),
        lib_root_for_task: Arc::new(library_root),
        missing_paths,
        start_time,
        silent,
    };
    let notification_task = start_notification_task(task_args);

    compile_work_queue(scan_root, work_queue, jobs, notify_tx.clone()).await?;

    drop(notify_tx);
    let _ = notification_task.await;

    let elapsed = start_time.elapsed().as_millis();
    if !silent {
        log::info!(
            "Update complete: {dirty_count} updated, {missing_count} removed. Finished in {elapsed}ms."
        );
    }

    let final_cache = Arc::try_unwrap(cache_arc).unwrap().into_inner();
    let _ = fs::write(&lua_hash_file, &lua_hash);
    save_cache(&final_cache, &cache_file)?;

    Ok(())
}

async fn notify_if_force_update(force: bool, is_full_library: bool) {
    if force && is_full_library {
        let client = reqwest::Client::new();
        let _ = client
            .post("http://127.0.0.1:8000/api/internal/notify_force_update")
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;
    }
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

async fn compile_work_queue(
    scan_root: PathBuf,
    work_queue: Vec<PathBuf>,
    jobs: Option<usize>,
    notify_tx: mpsc::Sender<compile::stream::AlbumUpdateSignal>,
) -> Result<()> {
    if !work_queue.is_empty() {
        let compile_options = compile::CompileOptions {
            target_path: scan_root,
            flags: vec!["default".to_string()],
            specific_albums: Some(work_queue),
            jobs,
            notify_tx: Some(notify_tx),
            compile_flags: compile::CompileFlags {
                mode: compile::CompileMode::Standard,
                target: compile::ExportTarget::File,
                pretty: false,
            },
        };
        compile::run(compile_options).await?;
    }
    Ok(())
}
