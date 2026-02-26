use anyhow::{Context, Result};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;

use crate::compile::{ExportTarget, manifest, verify};

pub struct StreamContext {
    pub albums: Vec<PathBuf>,
    pub config: Arc<Value>,
    pub project_root: Arc<PathBuf>,
    pub gen_cfg: Arc<Value>,
    pub active_flags: Arc<Vec<String>>,
    pub target: ExportTarget,
    pub jobs: Option<usize>,
    pub no_extensions: bool,
    pub notify_tx: Option<mpsc::Sender<PathBuf>>,
}

pub async fn run(child: Option<Child>, ctx: StreamContext) -> Result<()> {
    let (kernel_tx, kernel_rx) = mpsc::channel::<String>(32);
    let (direct_tx, mut direct_rx) = mpsc::channel::<Value>(512);

    let registry = ctx
        .config
        .get("compiler_registry")
        .and_then(Value::as_object)
        .map(|v| v.clone().into_iter().collect::<HashMap<String, Value>>())
        .unwrap_or_default();
    let registry_arc = Arc::new(registry);
    let notify_tx_arc = ctx.notify_tx.clone().map(Arc::new);

    let blocking_handle = spawn_manifest_builders(&ctx, kernel_tx, direct_tx);

    let notify_for_direct = notify_tx_arc.clone();
    let registry_for_direct = Arc::clone(&registry_arc);
    let target = ctx.target;

    let direct_consumer_handle = tokio::spawn(async move {
        while let Some(enriched) = direct_rx.recv().await {
            let notify = notify_for_direct.as_ref().map(Arc::clone);
            let reg = registry_for_direct.clone();
            tokio::task::spawn_blocking(move || {
                let _ = finalize_and_write(enriched, target, notify, &reg);
            });
        }
    });

    if let Some(mut child_proc) = child {
        run_kernel_bridge(
            &mut child_proc,
            kernel_rx,
            notify_tx_arc,
            registry_arc,
            target,
        )
        .await?;
    }

    let _ = blocking_handle.await;
    let _ = direct_consumer_handle.await;
    Ok(())
}

fn spawn_manifest_builders(
    ctx: &StreamContext,
    kernel_tx: mpsc::Sender<String>,
    direct_tx: mpsc::Sender<Value>,
) -> tokio::task::JoinHandle<()> {
    let albums = ctx.albums.clone();
    let config = Arc::clone(&ctx.config);
    let project_root = Arc::clone(&ctx.project_root);
    let gen_cfg = Arc::clone(&ctx.gen_cfg);
    let active_flags = Arc::clone(&ctx.active_flags);
    let jobs = ctx.jobs;
    let no_ext = ctx.no_extensions;

    tokio::task::spawn_blocking(move || {
        let threads = jobs.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(1)
        });
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        pool.install(|| {
            albums.par_iter().for_each(|album_root| {
                if let Ok((man, needs_ext)) = manifest::build(
                    album_root,
                    &project_root,
                    &config,
                    &gen_cfg,
                    &active_flags,
                    no_ext,
                ) {
                    if !needs_ext || no_ext {
                        let _ = direct_tx.blocking_send(man);
                    } else if let Ok(line) = serde_json::to_string(&man) {
                        let _ = kernel_tx.blocking_send(line);
                    }
                }
            });
        });
    })
}

async fn run_kernel_bridge(
    child: &mut Child,
    mut kernel_rx: mpsc::Receiver<String>,
    notify_tx: Option<Arc<mpsc::Sender<PathBuf>>>,
    registry: Arc<HashMap<String, Value>>,
    target: ExportTarget,
) -> Result<()> {
    let mut stdin = child.stdin.take().context("No kernel stdin")?;
    let stdout = child.stdout.take().context("No kernel stdout")?;

    let sender_handle = tokio::spawn(async move {
        while let Some(line) = kernel_rx.recv().await {
            let _ = stdin.write_all(line.as_bytes()).await;
            let _ = stdin.write_u8(b'\n').await;
        }
    });

    let receiver_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(enriched) = serde_json::from_str(&line) {
                let notify = notify_tx.as_ref().map(Arc::clone);
                let reg = registry.clone();
                tokio::task::spawn_blocking(move || {
                    let _ = finalize_and_write(enriched, target, notify, &reg);
                });
            }
        }
    });

    let _ = sender_handle.await;
    let _ = receiver_handle.await;
    let _ = child.wait().await;
    Ok(())
}

fn strip_empty_values(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| match v {
                Value::String(s) => !s.is_empty(),
                Value::Null => false,
                _ => true,
            });
            for v in map.values_mut() {
                strip_empty_values(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_empty_values(v);
            }
        }
        _ => {}
    }
}

fn finalize_and_write(
    mut enriched: Value,
    target: ExportTarget,
    notify_tx: Option<Arc<mpsc::Sender<PathBuf>>>,
    registry: &HashMap<String, Value>,
) -> Result<()> {
    let ctx = enriched.as_object_mut().map_or_else(
        || json!({}),
        |obj| obj.remove("ctx").unwrap_or_else(|| json!({})),
    );

    let harvest = ctx.get("harvest").cloned().unwrap_or_else(|| json!([]));
    let h_arr = harvest.as_array().map_or(&[][..], Vec::as_slice);
    let is_match = verify::calculate_file_tag_subset_match(&enriched, h_arr, registry);

    if let Some(a) = enriched.get_mut("album").and_then(Value::as_object_mut)
        && let Some(info) = a.get_mut("info").and_then(Value::as_object_mut)
    {
        info.insert("file_tag_subset_match".to_string(), json!(is_match));
    }

    strip_empty_values(&mut enriched);

    let album_root_str = ctx
        .get("paths")
        .and_then(|p| p.get("album_root"))
        .and_then(Value::as_str);
    if let Some(path) = album_root_str {
        let album_root = Path::new(path);
        let dest = album_root.join("metadata.lock.json");
        let content = serde_json::to_string_pretty(&enriched)?;
        if target == ExportTarget::Stdout {
            println!("{content}");
        } else {
            std::fs::write(dest, content)?;

            if let Some(tx_arc) = notify_tx {
                let root_clone = album_root.to_path_buf();
                let tx = (*tx_arc).clone();
                tokio::spawn(async move {
                    let _ = tx.send(root_clone).await;
                });
            }
        }
    }
    Ok(())
}
