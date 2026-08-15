use crate::compile::{build, utils, ExportTarget};
use anyhow::Result;
use libdale::error::DaleError;
use rayon::prelude::*;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct StreamContext {
    pub albums: Vec<PathBuf>,
    pub config: Arc<libdale::lua::ResolvedConfig>,
    pub target: ExportTarget,
    pub jobs: Option<usize>,
    pub ingest_tx: Option<tokio::sync::mpsc::Sender<crate::server::api::system::AlbumIngestPayload>>,
    pub active_writes: Option<Arc<Mutex<HashSet<PathBuf>>>>,
    pub silent: bool,
}

pub async fn run(ctx: StreamContext) -> Result<usize> {
    let (dtx, mut drx) = tokio::sync::mpsc::channel::<(Value, Option<Value>, HashSet<PathBuf>)>(512);
    let written_count = Arc::new(AtomicUsize::new(0));

    let build_handle = spawn_builders(&ctx, dtx);

    let target = ctx.target;
    let written_ref = Arc::clone(&written_count);
    let ingest_tx = ctx.ingest_tx;
    let active_writes = ctx.active_writes.clone();
    let silent = ctx.silent;

    let direct_handle = tokio::spawn(async move {
        while let Some((v, eval_res, deps)) = drx.recv().await {
            let written_inner = Arc::clone(&written_ref);
            let active_ref = active_writes.clone();
            let res = tokio::task::spawn_blocking(move || {
                finalize(v, eval_res, deps, target, active_ref.as_ref(), silent)
            })
            .await;
            if let Ok(Ok((written, payload))) = res {
                if written {
                    written_inner.fetch_add(1, Ordering::Relaxed);
                }
                if let Some(p) = payload
                    && let Some(ref tx) = ingest_tx
                {
                    let _ = tx.send(p).await;
                }
            }
        }
    });

    let _ = build_handle.await;
    let _ = direct_handle.await;

    Ok(written_count.load(Ordering::Relaxed))
}

fn spawn_builders(
    ctx: &StreamContext,
    dtx: tokio::sync::mpsc::Sender<(Value, Option<Value>, HashSet<PathBuf>)>,
) -> tokio::task::JoinHandle<()> {
    let albums = ctx.albums.clone();
    let cfg = Arc::clone(&ctx.config);
    let jobs = ctx.jobs;

    tokio::task::spawn_blocking(move || {
        let default_jobs =
            std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.unwrap_or(default_jobs))
            .build()
            .unwrap();
        pool.install(|| {
            albums.par_iter().for_each_init(
                || match libdale::lua::LuaEngine::new() {
                    Ok(engine) => match engine.evaluate_config(&cfg.path) {
                        Ok(_) => Some(engine),
                        Err(e) => {
                            log::error!("Failed to evaluate config for worker thread: {e}");
                            None
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to initialize Lua engine for worker thread: {e}");
                        None
                    }
                },
                |engine_opt, ar| {
                    let Some(engine) = engine_opt.as_ref() else {
                        log::error!("Skipping compilation of {} due to worker Lua engine initialization failure", ar.display());
                        return;
                    };
                    match build::build(ar, &cfg, engine) {
                        Ok(out) => {
                            let eval_res = engine.evaluate_album_logic(&out.lock_json).ok();
                            let _ = dtx.blocking_send((out.lock_json, eval_res, out.dependencies));
                        }
                        Err(e) => match e {
                            DaleError::ManifestIoError(_)
                            | DaleError::ManifestParseError { .. }
                            | DaleError::JsonParseError { .. }
                            | DaleError::InvalidFileExtension { .. }
                            | DaleError::InvalidManifestName { .. }
                            | DaleError::DuplicateManifestName { .. }
                            | DaleError::JsonError(_) => {
                                log::error!("SYSTEM FAILURE: {e}");
                            }
                            _ => {
                                log::warn!("VALIDATION REJECTED: {e}");
                            }
                        },
                    }
                },
            );
        });
    })
}

fn finalize(
    mut v: Value,
    eval_res: Option<Value>,
    deps: HashSet<PathBuf>,
    target: ExportTarget,
    active_writes: Option<&Arc<Mutex<HashSet<PathBuf>>>>,
    silent: bool,
) -> Result<(bool, Option<crate::server::api::system::AlbumIngestPayload>)> {
    let artist = v
        .get("album")
        .and_then(|a| a.get("albumartist"))
        .and_then(Value::as_str)
        .unwrap_or("Unknown")
        .to_string();
    let album = v
        .get("album")
        .and_then(|a| a.get("album"))
        .and_then(Value::as_str)
        .unwrap_or("Unknown")
        .to_string();
    let album_id = v
        .get("album")
        .and_then(|a| a.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let ctx = v
        .as_object_mut()
        .and_then(|o| o.remove("ctx"))
        .unwrap_or_else(|| json!({}));

    utils::strip_empty_values(&mut v);

    let album_root_str = ctx
        .get("paths")
        .and_then(|p| p.get("album_root"))
        .and_then(Value::as_str);

    if let Some(path) = album_root_str {
        let album_root = Path::new(path);

        let content = serde_json::to_string_pretty(&v)?;

        if target == ExportTarget::Stdout {
            println!("{content}");
            return Ok((false, None));
        }

        let lock_path = album_root.join("album.lock.json");
        let should_write =
            std::fs::read_to_string(&lock_path).map_or(true, |existing| existing != content);

        let payload = crate::server::api::system::AlbumIngestPayload {
            id: album_id,
            artist: artist.clone(),
            album: album.clone(),
            lock_json: content.clone(),
            eval_res,
            dependencies: deps.into_iter().collect(),
            modified: should_write,
        };

        if should_write {
            if let Some(aw) = active_writes
                && let Ok(mut active) = aw.lock()
            {
                let canon = lock_path.canonicalize().unwrap_or_else(|_| {
                    album_root.canonicalize().map_or_else(
                        |_| lock_path.clone(),
                        |parent_canon| parent_canon.join("album.lock.json"),
                    )
                });
                active.insert(canon);
                active.insert(lock_path.clone());
            } else if !silent {
                log::info!("Updated: {artist} - {album}");
            }
            std::fs::write(&lock_path, content)?;
            return Ok((true, Some(payload)));
        }

        return Ok((false, Some(payload)));
    }
    Ok((false, None))
}
