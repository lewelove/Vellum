use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::compile::nix::get_nix_env;

pub mod kernel;
pub mod manifest;
pub mod nix;
pub mod resolve;
pub mod scan;
pub mod stream;
pub mod verify;

pub async fn run(target_path: PathBuf, json_output: bool, intermediary: bool, pretty: bool) -> Result<()> {
    let project_root = scan::find_project_root().context("Project root not found")?;
    let config_content = fs::read_to_string(project_root.join("config.toml"))?;
    let config_toml: toml::Value = toml::from_str(&config_content)?;

    let scan_depth = config_toml.get("compiler")
        .and_then(|c| c.get("scan_depth"))
        .and_then(|v| v.as_integer())
        .unwrap_or(4) as usize;

    let albums = scan::find_target_albums(&target_path, scan_depth)?;
    if albums.is_empty() {
        log::info!("No metadata.toml files found.");
        return Ok(());
    }

    let config_json = serde_json::to_value(&config_toml)?;
    let gen_cfg = config_json.get("generate").cloned().unwrap_or(json!({}));

    if intermediary {
        for album_root in albums {
            let man = manifest::build(&album_root, &project_root, &config_json, &gen_cfg)?;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&man)?);
            } else {
                println!("{}", serde_json::to_string(&man)?);
            }
        }
        return Ok(());
    }

    let home = dirs::home_dir().context("No home dir")?;
    
    let ext_cfg = config_toml.get("extensions");
    let explicit_flake = ext_cfg
        .and_then(|c| {
            let folder = c.get("folder")?.as_str()?;
            let flake = c.get("flake")?.as_str()?;
            Some(PathBuf::from(folder).join(flake))
        });

    let mut nix_env = get_nix_env(&project_root, explicit_flake)?;
    nix_env.insert("HOME".to_string(), home.to_string_lossy().to_string());

    log::info!("Compiling {} albums...", albums.len());

    let child = kernel::spawn(&config_toml, &project_root, &nix_env).await?;

    stream::run(
        child,
        albums,
        Arc::new(config_json),
        Arc::new(project_root),
        Arc::new(gen_cfg),
        json_output,
    ).await
}
