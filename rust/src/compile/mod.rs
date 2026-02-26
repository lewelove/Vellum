use anyhow::{
    Context,
    Result,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::config::AppConfig;
use crate::compile::nix::get_nix_env;

pub mod kernel;
pub mod manifest;
pub mod native_extensions;
pub mod nix;
pub mod resolve;
pub mod scan;
pub mod stream;
pub mod verify;

pub async fn run(
    target_path: PathBuf,
    stdout_output: bool,
    intermediary: bool,
    pretty: bool,
    mut flags: Vec<String>,
    specific_albums: Option<Vec<PathBuf>>,
    jobs: Option<usize>,
    no_extensions: bool,
    notify_tx: Option<mpsc::Sender<PathBuf>>,
) -> Result<()> {
    let (config, raw_toml, config_path) = AppConfig::load()
        .context("Failed to load application configuration")?;

    let project_root = config_path.parent()
        .context("Failed to determine project root from config path")?
        .to_path_buf();

    if !flags.contains(&"default".to_string()) {
        flags.push(
            "default".to_string()
        );
    }

    let albums = if let Some(list) = specific_albums {
        list
    } else {
        let scan_depth = config.compiler.as_ref()
            .and_then(|c| c.scan_depth)
            .unwrap_or(4);
        scan::find_target_albums(&target_path, scan_depth)?
    };

    if albums.is_empty() {
        log::info!("No target albums for compilation.");
        return Ok(());
    }

    let config_json = serde_json::to_value(&raw_toml)?;
    let gen_cfg = config_json.get("generate").cloned().unwrap_or(json!({}));
    let active_flags = Arc::new(flags);

    if intermediary {
        for album_root in albums {
            let (man, _) = manifest::build(
                &album_root,
                &project_root,
                &config_json,
                &gen_cfg,
                &active_flags,
                no_extensions,
            )?;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&man)?);
            } else {
                println!("{}", serde_json::to_string(&man)?);
            }
        }
        return Ok(());
    }

    let registry = config_json.get("compiler_registry").and_then(|v| v.as_object());
    let has_extensions = registry.map(|r| {
        r.values().any(|v| v.get("provider").and_then(|s| s.as_str()) == Some("extension"))
    }).unwrap_or(false);

    let effective_no_extensions = no_extensions || !has_extensions;

    if effective_no_extensions {
        log::info!("Compiling {} albums (Native Only)...", albums.len());
        return stream::run(
            None,
            albums,
            Arc::new(config_json),
            Arc::new(project_root),
            Arc::new(gen_cfg),
            active_flags,
            stdout_output,
            jobs,
            true,
            notify_tx,
        ).await;
    }

    let home = dirs::home_dir().context("No home dir")?;
    
    let explicit_flake = config.extensions.as_ref().map(|ext| {
        PathBuf::from(&ext.folder).join(&ext.flake)
    });

    let mut nix_env = get_nix_env(&project_root, explicit_flake)?;
    nix_env.insert(
        "HOME".to_string(),
        home.to_string_lossy().to_string()
    );

    log::info!("Compiling {} albums...", albums.len());

    let child = kernel::spawn(
        &serde_json::from_value(config_json.clone())?,
        &project_root,
        &nix_env,
    )?;

    stream::run(
        Some(child),
        albums,
        Arc::new(config_json),
        Arc::new(project_root),
        Arc::new(gen_cfg),
        active_flags,
        stdout_output,
        jobs,
        false,
        notify_tx,
    ).await
}
