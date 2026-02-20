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
    mut child: Child,
    albums: Vec<PathBuf>,
    config: Arc<Value>,
    project_root: Arc<PathBuf>,
    gen_cfg: Arc<Value>,
    json_output: bool,
) -> Result<()> {
    let mut stdin = child.stdin.take().context("Failed to open Kernel stdin")?;
    let stdout = child.stdout.take().context("Failed to open Kernel stdout")?;

    // Channel for streaming generated manifests from the Rayon threadpool to the Async Sender
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // 1. Manifest Generation (CPU Bound)
    // We spawn this on a blocking thread to avoid stalling the async runtime.
    tokio::task::spawn_blocking(move || {
        albums.par_iter().for_each(|album_root| {
            // manifest::build now uses internal optimizations (lazy image loading, etc.)
            match manifest::build(album_root, &project_root, &config, &gen_cfg) {
                Ok(man) => {
                    if let Ok(line) = serde_json::to_string(&man) {
                        // Blocking send is fine here as we are in a blocking thread,
                        // but since we are bridging to async mpsc, we use blocking_send.
                        let _ = tx.blocking_send(line);
                    }
                }
                Err(e) => {
                    log::error!("Manifest Build Failed for {:?}: {}", album_root, e);
                }
            }
        });
    });

    // 2. Sender Task (Async)
    // Feeds the kernel stdin
    let sender_handle = tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if let Err(e) = stdin.write_all(line.as_bytes()).await {
                log::error!("Failed to write to kernel stdin: {}", e);
                break;
            }
            if let Err(e) = stdin.write_u8(b'\n').await {
                log::error!("Failed to write newline to kernel: {}", e);
                break;
            }
        }
        // Explicitly drop stdin to close the pipe, signaling EOF to the kernel
        drop(stdin);
    });

    // 3. Receiver Task (Async)
    // Reads kernel stdout, verifies, and writes to disk
    let receiver_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() { continue; }

            // Deserialize directly into Value to check context
            let mut enriched: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Kernel JSON Error: {}", e);
                    continue;
                }
            };
            
            // Extract original context for verification
            // We use .take() to avoid cloning the massive object if possible, 
            // but since it's inside a Map, we might need to remove it.
            let ctx = if let Some(obj) = enriched.as_object_mut() {
                obj.remove("ctx").unwrap_or(json!({}))
            } else {
                json!({})
            };

            let harvest = ctx.get("harvest").cloned().unwrap_or(json!([]));
            let h_arr = harvest.as_array().map(|v| v.as_slice()).unwrap_or(&[]);
            
            let is_match = verify::calculate_file_tag_subset_match(&enriched, h_arr);
            
            if let Some(a) = enriched.get_mut("album").and_then(|v| v.as_object_mut()) {
                a.insert("file_tag_subset_match".to_string(), json!(is_match));
            }

            let album_root_str = ctx.get("paths").and_then(|p| p.get("album_root")).and_then(|s| s.as_str());

            if let Some(path) = album_root_str {
                let dest = PathBuf::from(path).join("metadata.lock.json");
                if json_output {
                    println!("{}", serde_json::to_string_pretty(&enriched).unwrap_or_default());
                } else {
                    if let Err(e) = tokio::fs::write(dest, serde_json::to_string_pretty(&enriched).unwrap_or_default()).await {
                        log::error!("Failed to write lockfile: {}", e);
                    }
                }
            }
        }
    });

    // Await completion
    let _ = sender_handle.await;
    let _ = receiver_handle.await;
    let _ = child.wait().await;

    Ok(())
}
