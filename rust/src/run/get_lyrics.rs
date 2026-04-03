use crate::config::AppConfig;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub async fn run(
    config: &AppConfig,
    target_album: &Path,
    env_vars: &HashMap<String, String>,
) -> Result<()> {
    let script_path_str = config
        .run
        .as_ref()
        .and_then(|r| r.get("get-lyrics"))
        .cloned()
        .unwrap_or_else(|| "python/loose_scripts/fetch_lyrics.py".to_string());

    log::info!("Running get-lyrics on {}", target_album.display());

    let status = Command::new("python")
        .envs(env_vars)
        .arg(&script_path_str)
        .arg(target_album)
        .status()
        .context("Failed to execute get-lyrics script")?;

    if status.success() {
        log::info!("get-lyrics completed successfully. Triggering library update...");
        crate::update::run(Some(target_album.to_path_buf()), false, None).await?;
    } else {
        log::error!("get-lyrics script failed with status: {}", status);
    }

    Ok(())
}
