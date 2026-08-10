use crate::compile::{build, utils, ExportTarget};
use anyhow::Result;
use libdale::error::DaleError;
use rayon::prelude::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AlbumUpdateSignal {
    pub path: PathBuf,
    pub artist: String,
    pub album: String,
    pub lock_json: String,
}

pub struct StreamContext {
    pub albums: Vec<PathBuf>,
    pub config: Arc<libdale::lua::ResolvedConfig>,
    pub target: ExportTarget,
    pub jobs: Option<usize>,
    pub notify_tx: Option<mpsc::Sender<AlbumUpdateSignal>>,
}

pub async fn run(ctx: StreamContext) -> Result<()> {
    let (dtx, mut drx) = mpsc::channel::<Value>(512);

    let notify = ctx.notify_tx.clone().map(Arc::new);
    let build_handle = spawn_builders(&ctx, dtx);

    let d_notify = notify.clone();
    let target = ctx.target;
    let direct_handle = tokio::spawn(async move {
        while let Some(v) = drx.recv().await {
            let n = d_notify.as_ref().map(Arc::clone);
            tokio::task::spawn_blocking(move || {
                let _ = finalize(v, target, n);
            });
        }
    });

    let _ = build_handle.await;
    let _ = direct_handle.await;
    Ok(())
}

fn spawn_builders(ctx: &StreamContext, dtx: mpsc::Sender<Value>) -> tokio::task::JoinHandle<()> {
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
            albums.par_iter().for_each(|ar| {
                match build::build(ar, &cfg) {
                    Ok(man) => {
                        let _ = dtx.blocking_send(man);
                    }
                    Err(e) => match e {
                        DaleError::ManifestIoError(_)
                        | DaleError::ManifestParseError { .. }
                        | DaleError::JsonError(_) => {
                            log::error!("SYSTEM FAILURE: {e}");
                        }
                        _ => {
                            log::warn!("VALIDATION REJECTED: {e}");
                        }
                    },
                }
            });
        });
    })
}

fn finalize(
    mut v: Value,
    target: ExportTarget,
    notify_tx: Option<Arc<mpsc::Sender<AlbumUpdateSignal>>>,
) -> Result<()> {
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
        } else {
            let lock_path = album_root.join("album.lock.json");
            let should_write =
                std::fs::read_to_string(&lock_path).map_or(true, |existing| existing != content);

            if should_write {
                std::fs::write(lock_path, content.clone())?;
            }

            if let Some(tx_arc) = notify_tx {
                let root_clone = album_root.to_path_buf();
                let tx = (*tx_arc).clone();
                let lock_json = content;
                tokio::spawn(async move {
                    let _ = tx
                        .send(AlbumUpdateSignal {
                            path: root_clone,
                            artist,
                            album,
                            lock_json,
                        })
                        .await;
                });
            }
        }
    }
    Ok(())
}
