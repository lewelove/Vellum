pub mod get_cover_palette;
pub mod get_lyrics;

use vellum::config::AppConfig;
use vellum::utils::expand_path;
use anyhow::{Context, Result};
use mpd_client::Client;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::net::TcpStream;

pub async fn execute(cmd: String, path_arg: Option<String>, playing: bool, id_arg: Option<String>) -> Result<()> {
    let (config, _, _): (AppConfig, toml::Value, PathBuf) = AppConfig::load().context("Failed to load config")?;

    let mut env_vars = HashMap::new();
    if let Some(env_path) = &config.storage.environment {
        let expanded = expand_path(env_path);
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

    let target_album = if playing {
        get_playing_album(&config.storage.library_root).await?
    } else if let Some(id) = id_arg {
        expand_path(&config.storage.library_root).join(id)
    } else if let Some(p) = path_arg {
        expand_path(&p).canonicalize().unwrap_or_else(|_| expand_path(&p))
    } else {
        get_playing_album(&config.storage.library_root).await?
    };

    env_vars.insert(
        "ALBUM_PATH".to_string(),
        target_album.to_string_lossy().to_string(),
    );

    match cmd.as_str() {
        "get-cover-palette" => get_cover_palette::run(&config, &target_album).await,
        "get-lyrics" => get_lyrics::run(&config, &target_album, &env_vars).await,
        _ => {
            if let Some(script_path) = config.run.as_ref().and_then(|r: &HashMap<String, String>| r.get(&cmd)) {
                log::info!("Running script '{}' on {}", cmd, target_album.display());
                let status = std::process::Command::new("python")
                    .envs(&env_vars)
                    .arg(script_path)
                    .arg(&target_album)
                    .status()
                    .context("Failed to execute script")?;

                if status.success() {
                    log::info!("Script completed successfully. Triggering library update...");
                    crate::update::run(Some(target_album), false, None, false, false).await?;
                } else {
                    log::error!("Script failed with status: {status}");
                }
                Ok(())
            } else {
                anyhow::bail!("Unknown command and no script configured for '{cmd}'");
            }
        }
    }
}

pub async fn get_playing_album(lib_root: &str) -> Result<PathBuf> {
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

    let rel_path = song.song.url;
    let root = expand_path(lib_root);
    let full_path = root.join(rel_path);

    let album_dir = full_path
        .parent()
        .context("Invalid track path")?
        .to_path_buf();
        
    Ok(album_dir)
}
