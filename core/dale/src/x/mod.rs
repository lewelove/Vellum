pub mod builtin;

use anyhow::{Context, Result};
use libdale::utils::expand_path;
use mpd_client::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::net::TcpStream;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugMode {
    Enabled,
    Disabled,
}

pub struct TargetFlags {
    pub playing: bool,
    pub id: Option<String>,
    pub query: Option<String>,
    pub directory: Option<String>,
    pub recursive: Option<String>,
    pub library: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ResolvedAlbumTarget {
    pub id: String,
    pub path: PathBuf,
    pub lock: serde_json::Value,
}

pub fn load_env_vars_from_path(
    env_path: Option<&str>,
) -> std::collections::HashMap<String, String> {
    let mut env_vars = std::collections::HashMap::new();
    if let Some(path_str) = env_path {
        let expanded = expand_path(path_str);
        if let Ok(content) = std::fs::read_to_string(&expanded) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    env_vars.insert(
                        k.trim().to_string(),
                        v.trim().trim_matches(|c| c == '"' || c == '\'').to_string(),
                    );
                }
            }
        }
    }
    env_vars
}

async fn resolve_by_query(query_str: &str) -> Result<Vec<ResolvedAlbumTarget>> {
    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:8000/api/internal/query")
        .json(&serde_json::json!({ "query": query_str }))
        .send()
        .await
        .context("Failed to connect to the Dale server. Is it running?")?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        anyhow::bail!("Server rejected query: {err_text}");
    }

    let items: Vec<ResolvedAlbumTarget> =
        res.json().await.context("Invalid response from server")?;
    Ok(items)
}

async fn resolve_by_id(
    id: &str,
    music_directory: &Path,
) -> Result<Vec<ResolvedAlbumTarget>> {
    let client = reqwest::Client::new();
    let q_resp = client
        .post("http://127.0.0.1:8000/api/internal/query")
        .json(&serde_json::json!({ "query": id }))
        .send()
        .await;

    if let Ok(resp) = q_resp
        && resp.status().is_success()
        && let Ok(results) = resp.json::<Vec<ResolvedAlbumTarget>>().await
        && let Some(matched) = results.into_iter().find(|r| r.id == id)
    {
        return Ok(vec![matched]);
    }

    let mut results = Vec::new();
    for entry in WalkDir::new(music_directory)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_name() == "album.lock.json"
            && let Ok(content) = std::fs::read_to_string(entry.path())
            && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&content)
            && lock_json.pointer("/album/id").and_then(|v| v.as_str()) == Some(id)
            && let Some(parent) = entry.path().parent()
        {
            results.push(ResolvedAlbumTarget {
                id: id.to_string(),
                path: parent
                    .canonicalize()
                    .unwrap_or_else(|_| parent.to_path_buf()),
                lock: lock_json,
            });
            break;
        }
    }
    Ok(results)
}

async fn resolve_by_playing(music_directory: &Path) -> Result<Vec<ResolvedAlbumTarget>> {
    let playing_album_dir = get_playing_album(&music_directory.to_string_lossy()).await?;
    let lock_path = playing_album_dir.join("album.lock.json");
    if lock_path.exists()
        && let Ok(content) = std::fs::read_to_string(&lock_path)
        && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&content)
    {
        let id = lock_json
            .pointer("/album/id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        Ok(vec![ResolvedAlbumTarget {
            id,
            path: playing_album_dir
                .canonicalize()
                .unwrap_or(playing_album_dir),
            lock: lock_json,
        }])
    } else {
        Ok(Vec::new())
    }
}

fn resolve_by_scan(root: &Path) -> Vec<ResolvedAlbumTarget> {
    let mut results = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_name() == "album.lock.json"
            && let Ok(content) = std::fs::read_to_string(entry.path())
            && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(parent) = entry.path().parent()
        {
            let id = lock_json
                .pointer("/album/id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            results.push(ResolvedAlbumTarget {
                id,
                path: parent
                    .canonicalize()
                    .unwrap_or_else(|_| parent.to_path_buf()),
                lock: lock_json,
            });
        }
    }
    results
}

fn resolve_by_dir(dir: &Path) -> Vec<ResolvedAlbumTarget> {
    let p = expand_path(dir.to_str().unwrap_or("."));
    let lock_path = p.join("album.lock.json");
    if lock_path.exists()
        && let Ok(content) = std::fs::read_to_string(&lock_path)
        && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&content)
    {
        let id = lock_json
            .pointer("/album/id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        vec![ResolvedAlbumTarget {
            id,
            path: p.canonicalize().unwrap_or(p),
            lock: lock_json,
        }]
    } else {
        Vec::new()
    }
}

pub async fn resolve_target_albums(
    music_directory: &Path,
    target: &TargetFlags,
) -> Result<Vec<ResolvedAlbumTarget>> {
    if let Some(query_str) = &target.query {
        resolve_by_query(query_str).await
    } else if let Some(id) = &target.id {
        resolve_by_id(id, music_directory).await
    } else if target.playing {
        resolve_by_playing(music_directory).await
    } else if target.library {
        Ok(resolve_by_scan(music_directory))
    } else if let Some(dir) = &target.recursive {
        Ok(resolve_by_scan(&expand_path(dir)))
    } else {
        let dir_str = target.directory.as_deref().unwrap_or(".");
        Ok(resolve_by_dir(Path::new(dir_str)))
    }
}

async fn run_external_action(
    action_path: &Path,
    env_vars: &std::collections::HashMap<String, String>,
    payload_json: &serde_json::Value,
) -> Result<()> {
    let cmd = if action_path.extension().is_some_and(|e| e == "py") {
        "python"
    } else if action_path.extension().is_some_and(|e| e == "sh") {
        "sh"
    } else {
        action_path.to_str().unwrap()
    };

    let mut command = tokio::process::Command::new(cmd);
    command.envs(env_vars);
    if cmd == "python" || cmd == "sh" {
        command.arg(action_path);
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context(format!(
            "Failed to spawn action at {}",
            action_path.display()
        ))?;

    if let Some(mut stdin) = child.stdin.take() {
        let payload = serde_json::to_string(payload_json)?;
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(payload.as_bytes()).await;
        });
    }

    tokio::select! {
        res = child.wait() => {
            let status = res.context("Failed to wait on action")?;
            if !status.success() {
                log::error!("Action failed with status: {status}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            let _ = child.wait().await;
        }
    }
    Ok(())
}

pub async fn execute(
    name: String,
    target: TargetFlags,
    _debug: DebugMode,
    trailing_args: Vec<String>,
) -> Result<()> {
    let name_key = name.replace('-', "_");
    let config = libdale::lua::ResolvedConfig::load().context("Failed to load config")?;

    let music_directory = expand_path(&config.app.storage.music_directory)
        .canonicalize()
        .unwrap_or_else(|_| expand_path(&config.app.storage.music_directory));

    let resolved_targets = resolve_target_albums(&music_directory, &target).await?;

    let albums_payload: Vec<serde_json::Value> = resolved_targets
        .iter()
        .map(|t| {
            serde_json::json!({
                "path": t.path.to_string_lossy(),
                "lock": t.lock
            })
        })
        .collect();

    let action_cfg_opt = config.actions.get(&name_key).cloned();
    let config_json = serde_json::to_value(&config.app)?;
    let action_config = action_cfg_opt
        .as_ref()
        .map(|c| c.config.clone())
        .unwrap_or_default();

    let combined_json = serde_json::json!({
        "albums": albums_payload,
        "config": {
            "dale": config_json,
            "action": action_config
        },
        "options": trailing_args.join(" ")
    });

    if name == "intermediary" {
        let pretty_json = serde_json::to_string_pretty(&combined_json)?;
        println!("{pretty_json}");
        return Ok(());
    }

    if let Some(r_str) = action_cfg_opt.as_ref().and_then(|a| a.run.clone()) {
        let action_path = PathBuf::from(&r_str);

        if action_path.exists() {
            let env_vars =
                load_env_vars_from_path(config.app.storage.environment.as_deref());
            run_external_action(&action_path, &env_vars, &combined_json).await?;
            return Ok(());
        }
        anyhow::bail!(
            "Action '{name}' script not found at path: {}",
            action_path.display()
        );
    }

    let mut executed_builtin = false;
    if name_key == "open_config_in_terminal" || resolved_targets.is_empty() {
        let dummy_path = Path::new("");
        if builtin::execute_builtin(&name_key, dummy_path, &action_config)? {
            executed_builtin = true;
        }
    } else {
        for target_item in &resolved_targets {
            if builtin::execute_builtin(&name_key, &target_item.path, &action_config)? {
                executed_builtin = true;
            }
        }
    }

    if !executed_builtin {
        anyhow::bail!(
            "Action '{name}' is not declared in configuration and no built-in exists."
        );
    }

    Ok(())
}

pub async fn get_playing_track_url() -> Result<String> {
    let host = std::env::var("MPD_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("MPD_PORT").unwrap_or_else(|_| "6600".to_string());
    let addr = format!("{host}:{port}");

    let stream = TcpStream::connect(&addr)
        .await
        .context("Failed to connect to MPD")?;
    let (client, _) = Client::connect(stream)
        .await
        .context("Failed to initialize MPD client")?;

    let current_song = client.command(mpd_client::commands::CurrentSong).await?;
    let song = current_song.context("No song is currently playing")?;
    Ok(song.song.url)
}

pub async fn get_playing_album(lib_root: &str) -> Result<PathBuf> {
    let rel_path = get_playing_track_url().await?;
    let clean_rel = rel_path.trim_start_matches('/');
    let root = expand_path(lib_root)
        .canonicalize()
        .unwrap_or_else(|_| expand_path(lib_root));
    let full_path = root.join(clean_rel);

    let mut curr = full_path
        .parent()
        .map(Path::to_path_buf)
        .context("Invalid track path")?;

    while curr.starts_with(&root) && curr != root {
        if curr.join("metadata.toml").exists() || curr.join("album.lock.json").exists() {
            return Ok(curr);
        }
        let Some(parent) = curr.parent() else {
            break;
        };
        curr = parent.to_path_buf();
    }

    full_path
        .parent()
        .map(Path::to_path_buf)
        .context("Invalid track path")
}
