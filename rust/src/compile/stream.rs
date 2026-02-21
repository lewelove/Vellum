use anyhow::{Context, Result};
use rayon::prelude::*;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;

use crate::compile::{manifest, verify};

pub async fn run(
    child: Option<Child>,
    albums: Vec<PathBuf>,
    config: Arc<Value>,
    project_root: Arc<PathBuf>,
    gen_cfg: Arc<Value>,
    active_flags: Arc<Vec<String>>,
    stdout_output: bool,
    jobs: Option<usize>,
    no_extensions: bool,
    notify_tx: Option<mpsc::Sender<PathBuf>>,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<String>(32);

    let albums_clone = albums.clone();
    let config_clone = Arc::clone(&config);
    let project_root_clone = Arc::clone(&project_root);
    let gen_cfg_clone = Arc::clone(&gen_cfg);
    let active_flags_clone = Arc::clone(&active_flags);
    let notify_tx_arc = notify_tx.map(Arc::new);

    let notify_tx_for_blocking = notify_tx_arc.clone();
    let blocking_handle = tokio::task::spawn_blocking(move || {
        let default_parallelism = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.unwrap_or(default_parallelism))
            .build()
            .unwrap();

        pool.install(|| {
            albums_clone.par_iter().for_each(|album_root| {
                match manifest::build(album_root, &project_root_clone, &config_clone, &gen_cfg_clone, &active_flags_clone, no_extensions) {
                    Ok(man) => {
                        let requires_python = man.get("requires_python").and_then(|v| v.as_bool()).unwrap_or(true);
                        
                        if !requires_python {
                            let _ = finalize_and_write(man, stdout_output, notify_tx_for_blocking.as_ref().map(|a| Arc::clone(a)));
                        } else if let Ok(line) = serde_json::to_string(&man) {
                            let _ = tx.blocking_send(line);
                        }
                    }
                    Err(e) => {
                        log::error!("Manifest Build Failed for {:?}: {}", album_root, e);
                    }
                }
            });
        });
    });

    if let Some(mut child_proc) = child {
        let mut stdin = child_proc.stdin.take().context("Failed to open Kernel stdin")?;
        let stdout = child_proc.stdout.take().context("Failed to open Kernel stdout")?;

        let notify_tx_for_receiver = notify_tx_arc.clone();
        let sender_handle = tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                let _ = stdin.write_all(line.as_bytes()).await;
                let _ = stdin.write_u8(b'\n').await;
            }
            drop(stdin);
        });

        let receiver_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() { continue; }
                if let Ok(enriched) = serde_json::from_str(&line) {
                    let _ = finalize_and_write(enriched, stdout_output, notify_tx_for_receiver.as_ref().map(|a| Arc::clone(a)));
                }
            }
        });

        let _ = sender_handle.await;
        let _ = receiver_handle.await;
        let _ = child_proc.wait().await;
    }

    let _ = blocking_handle.await;

    Ok(())
}

fn strip_empty_values(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| {
                match v {
                    Value::String(s) => !s.is_empty(),
                    Value::Null => false,
                    _ => true,
                }
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

fn finalize_and_write(mut enriched: Value, stdout_output: bool, notify_tx: Option<Arc<mpsc::Sender<PathBuf>>>) -> Result<()> {
    let ctx = if let Some(obj) = enriched.as_object_mut() {
        obj.remove("ctx").unwrap_or(json!({}))
    } else {
        json!({})
    };

    let harvest = ctx.get("harvest").cloned().unwrap_or(json!([]));
    let h_arr = harvest.as_array().map(|v| v.as_slice()).unwrap_or(&[]);
    let is_match = verify::calculate_file_tag_subset_match(&enriched, h_arr);
    
    if let Some(a) = enriched.get_mut("album").and_then(|v| v.as_object_mut()) {
        a.insert(
            "file_tag_subset_match".to_string(),
            json!(is_match)
        );
    }

    strip_empty_values(&mut enriched);

    let album_root_str = ctx.get("paths").and_then(|p| p.get("album_root")).and_then(|s| s.as_str());

    if let Some(path) = album_root_str {
        let album_root = PathBuf::from(path);
        let dest = album_root.join("metadata.lock.json");
        let content = serde_json::to_string_pretty(&enriched)?;
        if stdout_output {
            println!("{}", content);
        } else {
            std::fs::write(dest, content)?;
            if let Some(tx) = notify_tx {
                let _ = tx.blocking_send(album_root);
            }
        }
    }
    Ok(())
}
