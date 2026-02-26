use anyhow::{Context, Result};
use rayon::prelude::*;
use serde_json::{json, Value};
use std::path::PathBuf;
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
    // Channel for Kernel Processing (Legacy Pipeline)
    let (kernel_tx, mut kernel_rx) = mpsc::channel::<String>(32);
    
    // Channel for Direct Processing (New Native Pipeline)
    // Buffered to prevent Rayon threads from blocking on I/O
    let (direct_tx, mut direct_rx) = mpsc::channel::<Value>(512);

    let albums_clone = albums.clone();
    let config_clone = Arc::clone(&config);
    let project_root_clone = Arc::clone(&project_root);
    let gen_cfg_clone = Arc::clone(&gen_cfg);
    let active_flags_clone = Arc::clone(&active_flags);
    let notify_tx_arc = notify_tx.map(Arc::new);

    let registry = config.get("compiler_registry")
        .and_then(|v| v.as_object())
        .map(|v| v.clone().into_iter().collect::<HashMap<String, Value>>())
        .unwrap_or_default();
    let registry_arc = Arc::new(registry);
    
    // 1. PRODUCER: Rayon Thread Pool
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
                    Ok((man, needs_external)) => {
                        if !needs_external || no_extensions {
                            // Direct path: Send JSON object to direct consumer
                            let _ = direct_tx.blocking_send(man);
                        } else if let Ok(line) = serde_json::to_string(&man) {
                            // Kernel path: Send JSON string to kernel stdin
                            let _ = kernel_tx.blocking_send(line);
                        }
                    }
                    Err(e) => {
                        log::error!("Manifest Build Failed for {:?}: {}", album_root, e);
                    }
                }
            });
        });
    });

    let notify_tx_for_direct = notify_tx_arc.clone();
    let registry_for_direct = Arc::clone(&registry_arc);

    // 2. CONSUMER A: Direct Path (Native I/O Writer)
    let direct_consumer_handle = tokio::spawn(async move {
        while let Some(enriched) = direct_rx.recv().await {
            let notify = notify_tx_for_direct.as_ref().map(|a| Arc::clone(a));
            let reg = registry_for_direct.clone();
            
            // Offload disk I/O to a blocking thread to avoid stalling the async runtime
            // Do NOT await here; allow parallelism for disk operations.
            tokio::task::spawn_blocking(move || {
                let _ = finalize_and_write(enriched, stdout_output, notify, &reg);
            });
        }
    });

    // 3. CONSUMER B: Kernel Path (External Process)
    if let Some(mut child_proc) = child {
        let mut stdin = child_proc.stdin.take().context("Failed to open Kernel stdin")?;
        let stdout = child_proc.stdout.take().context("Failed to open Kernel stdout")?;

        let notify_tx_for_receiver = notify_tx_arc.clone();
        let registry_for_receiver = Arc::clone(&registry_arc);
        
        // Feed stdin
        let sender_handle = tokio::spawn(async move {
            while let Some(line) = kernel_rx.recv().await {
                let _ = stdin.write_all(line.as_bytes()).await;
                let _ = stdin.write_u8(b'\n').await;
            }
            drop(stdin);
        });

        // Read stdout
        let receiver_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() { continue; }
                if let Ok(enriched) = serde_json::from_str(&line) {
                    let notify = notify_tx_for_receiver.as_ref().map(|a| Arc::clone(a));
                    let reg = registry_for_receiver.clone();
                    
                    // Do NOT await here either.
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
    let ctx = if let Some(obj) = enriched.as_object_mut() {
        obj.remove("ctx").unwrap_or(json!({}))
    } else {
        json!({})
    };

    let harvest = ctx.get("harvest").cloned().unwrap_or(json!([]));
    let h_arr = harvest.as_array().map(|v| v.as_slice()).unwrap_or(&[]);
    let is_match = verify::calculate_file_tag_subset_match(&enriched, h_arr, registry);
    
    if let Some(a) = enriched.get_mut("album").and_then(|v| v.as_object_mut()) {
        if let Some(info) = a.get_mut("info").and_then(|v| v.as_object_mut()) {
            info.insert("file_tag_subset_match".to_string(), json!(is_match));
        }
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
            
            if let Some(tx_arc) = notify_tx {
                let root_clone = album_root.clone();
                let tx = (*tx_arc).clone();
                tokio::spawn(async move {
                    let _ = tx.send(root_clone).await;
                });
            }
        }
    }
    Ok(())
}
