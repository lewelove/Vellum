use crate::compile::{ExportTarget, builder, engine::verify};
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;

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
    let (ktx, krx) = mpsc::channel::<String>(32);
    let (dtx, mut drx) = mpsc::channel::<Value>(512);

    let reg = Arc::new(
        ctx.config
            .get("compiler_registry")
            .and_then(Value::as_object)
            .map(|v| v.clone().into_iter().collect::<HashMap<String, Value>>())
            .unwrap_or_default(),
    );

    let notify = ctx.notify_tx.clone().map(Arc::new);
    let build_handle = spawn_builders(&ctx, ktx, dtx);

    let d_notify = notify.clone();
    let d_reg = reg.clone();
    let target = ctx.target;
    let direct_handle = tokio::spawn(async move {
        while let Some(v) = drx.recv().await {
            let n = d_notify.as_ref().map(Arc::clone);
            let r = d_reg.clone();
            tokio::task::spawn_blocking(move || {
                let _ = finalize(v, target, n, &r);
            });
        }
    });

    if let Some(mut proc) = child {
        run_bridge(&mut proc, krx, notify, reg, target).await?;
    }

    let _ = build_handle.await;
    let _ = direct_handle.await;
    Ok(())
}

fn spawn_builders(
    ctx: &StreamContext,
    ktx: mpsc::Sender<String>,
    dtx: mpsc::Sender<Value>,
) -> tokio::task::JoinHandle<()> {
    let albums = ctx.albums.clone();
    let cfg = Arc::clone(&ctx.config);
    let root = Arc::clone(&ctx.project_root);
    let gcfg = Arc::clone(&ctx.gen_cfg);
    let flags = Arc::clone(&ctx.active_flags);
    let jobs = ctx.jobs;
    let no_ext = ctx.no_extensions;

    tokio::task::spawn_blocking(move || {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.unwrap_or(1))
            .build()
            .unwrap();
        pool.install(|| {
            albums.par_iter().for_each(|ar| {
                if let Ok((man, needs_ext)) = builder::build(ar, &root, &cfg, &gcfg, &flags, no_ext)
                {
                    if !needs_ext || no_ext {
                        let _ = dtx.blocking_send(man);
                    } else if let Ok(l) = serde_json::to_string(&man) {
                        let _ = ktx.blocking_send(l);
                    }
                }
            });
        });
    })
}

async fn run_bridge(
    child: &mut Child,
    mut krx: mpsc::Receiver<String>,
    notify: Option<Arc<mpsc::Sender<PathBuf>>>,
    reg: Arc<HashMap<String, Value>>,
    target: ExportTarget,
) -> Result<()> {
    let mut stdin = child.stdin.take().context("No stdin")?;
    let stdout = child.stdout.take().context("No stdout")?;
    let sender = tokio::spawn(async move {
        while let Some(l) = krx.recv().await {
            let _ = stdin.write_all(l.as_bytes()).await;
            let _ = stdin.write_u8(b'\n').await;
        }
    });
    let receiver = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(l)) = lines.next_line().await {
            if let Ok(v) = serde_json::from_str(&l) {
                let n = notify.as_ref().map(Arc::clone);
                let r = reg.clone();
                tokio::task::spawn_blocking(move || {
                    let _ = finalize(v, target, n, &r);
                });
            }
        }
    });
    let _ = sender.await;
    let _ = receiver.await;
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

fn finalize(
    mut v: Value,
    target: ExportTarget,
    notify_tx: Option<Arc<mpsc::Sender<PathBuf>>>,
    registry: &HashMap<String, Value>,
) -> Result<()> {
    let ctx = v
        .as_object_mut()
        .and_then(|o| o.remove("ctx"))
        .unwrap_or_else(|| json!({}));
    let harvest = ctx.get("harvest").cloned().unwrap_or_else(|| json!([]));
    let h_arr = harvest.as_array().map_or(&[][..], Vec::as_slice);

    let is_match = verify::calculate_file_tag_subset_match(&v, h_arr, registry);
    if let Some(info) = v
        .get_mut("album")
        .and_then(|a| a.get_mut("info"))
        .and_then(|i| i.as_object_mut())
    {
        info.insert("file_tag_subset_match".to_string(), json!(is_match));
    }

    strip_empty_values(&mut v);

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
            std::fs::write(album_root.join("metadata.lock.json"), content)?;
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
