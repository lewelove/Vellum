use anyhow::{Context, Result};
use rayon::prelude::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;
use std::collections::HashMap;

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
    let (kernel_tx, mut kernel_rx) = mpsc::channel::<String>(32);
    let (direct_tx, mut direct_rx) = mpsc::channel::<Value>(512);

    let albums_clone = albums.clone();
    let config_clone = Arc::clone(&config);
    let project_root_clone = Arc::clone(&project_root);
    let gen_cfg_clone = Arc::clone(&gen_cfg);
    let active_flags_clone = Arc::clone(&active_flags);
    let notify_tx_arc = notify_tx.map(Arc::new);

    let registry = config.get("compiler_registry")
        .and_then(Value::as_object)
        .map(|v| v.clone().into_iter().collect::<HashMap<String, Value>>())
        .unwrap_or_default();
    let registry_arc = Arc::new(registry);
    
    let blocking_handle = tokio::task::spawn_blocking(move || {
        let default_parallelism = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1);

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.unwrap_or(default_parallelism))
            .build()
            .unwrap();

        pool.install(|| {
            albums_clone.par_iter().for_each(|album_root| {
                match manifest::build(album_root, &project_root_clone, &config_clone, &gen_cfg_clone, &active_flags_clone, no_extensions) {
                    Ok((man, needs_external)) => {
                        if !needs_external || no_extensions {
                            let _ = direct_tx.blocking_send(man);
                        } else if let Ok(line) = serde_json::to_string(&man) {
                            let _ = kernel_tx.blocking_send(line);
                        }
                    }
                    Err(e) => {
                        log::error!("Manifest Build Failed for {album_root:?}: {e}");
                    }
                }
            });
        });
    });

    let notify_tx_for_direct = notify_tx_arc.clone();
    let registry_for_direct = Arc::clone(&registry_arc);

    let direct_consumer_handle = tokio::spawn(async move {
        while let Some(enriched) = direct_rx.recv().await {
            let notify = notify_tx_for_direct.as_ref().map(Arc::clone);
            let reg = registry_for_direct.clone();
            
            tokio::task::spawn_blocking(move || {
                let _ = finalize_and_write(enriched, stdout_output, notify, &reg);
            });
        }
    });

    if let Some(mut child_proc) = child {
        let mut stdin = child_proc.stdin.take().context("Failed to open Kernel stdin")?;
        let stdout = child_proc.stdout.take().context("Failed to open Kernel stdout")?;

        let notify_tx_for_receiver = notify_tx_arc.clone();
        let registry_for_receiver = Arc::clone(&registry_arc);
        
        let sender_handle = tokio::spawn(async move {
            while let Some(line) = kernel_rx.recv().await {
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
                    let notify = notify_tx_for_receiver.as_ref().map(Arc::clone);
                    let reg = registry_for_receiver.clone();
                    
                    tokio::task::spawn_blocking(move || {
                        let _ = finalize_and_write(enriched, stdout_output, notify, &reg);
                    });
                }
            }
        });

        let _ = sender_handle.await;
        let _ = receiver_handle.await;
        let _ = child_proc.wait().await;
    }

    let _ = blocking_handle.await;
    let _ = direct_consumer_handle.await;
    
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

fn finalize_and_write(mut enriched: Value, stdout_output: bool, notify_tx: Option<Arc<mpsc::Sender<PathBuf>>>, registry: &HashMap<String, Value>) -> Result<()> {
    let ctx = enriched.as_object_mut().map_or_else(|| json!({}), |obj| obj.remove("ctx").unwrap_or_else(|| json!({})));

    let harvest = ctx.get("harvest").cloned().unwrap_or_else(|| json!([]));
    let h_arr = harvest.as_array().map_or(&[][..], Vec::as_slice);
    let is_match = verify::calculate_file_tag_subset_match(&enriched, h_arr, registry);
    
    if let Some(a) = enriched.get_mut("album").and_then(Value::as_object_mut)
        && let Some(info) = a.get_mut("info").and_then(Value::as_object_mut) {
        info.insert("file_tag_subset_match".to_string(), json!(is_match));
    }

    strip_empty_values(&mut enriched);

    let album_root_str = ctx.get("paths").and_then(|p| p.get("album_root")).and_then(Value::as_str);
    if let Some(path) = album_root_str {
        let album_root = Path::new(path);
        let dest = album_root.join("metadata.lock.json");
        let content = serde_json::to_string_pretty(&enriched)?;
        if stdout_output {
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
