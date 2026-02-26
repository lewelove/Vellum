use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::compile::nix::get_nix_env;
use crate::compile::stream::StreamContext;
use crate::config::AppConfig;

pub mod kernel;
pub mod manifest;
pub mod native_extensions;
pub mod nix;
pub mod resolve;
pub mod scan;
pub mod stream;
pub mod verify;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    Standard,
    Intermediary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportTarget {
    File,
    Stdout,
}

pub struct CompileFlags {
    pub mode: CompileMode,
    pub target: ExportTarget,
    pub pretty: bool,
    pub no_extensions: bool,
}

pub struct CompileOptions {
    pub target_path: PathBuf,
    pub flags: Vec<String>,
    pub specific_albums: Option<Vec<PathBuf>>,
    pub jobs: Option<usize>,
    pub notify_tx: Option<mpsc::Sender<PathBuf>>,
    pub compile_flags: CompileFlags,
}

pub async fn run(mut options: CompileOptions) -> Result<()> {
    let (config, raw_toml, config_path) =
        AppConfig::load().context("Failed to load application configuration")?;

    let project_root = config_path
        .parent()
        .context("Failed to determine project root from config path")?
        .to_path_buf();

    if !options.flags.contains(&"default".to_string()) {
        options.flags.push("default".to_string());
    }

    let albums = if let Some(list) = options.specific_albums {
        list
    } else {
        let scan_depth = config.compiler.as_ref().and_then(|c| c.scan_depth).unwrap_or(4);
        scan::find_target_albums(&options.target_path, scan_depth)
    };

    if albums.is_empty() {
        log::info!("No target albums for compilation.");
        return Ok(());
    }

    let config_json = serde_json::to_value(&raw_toml)?;
    let gen_cfg = config_json.get("generate").cloned().unwrap_or_else(|| json!({}));
    let active_flags = Arc::new(options.flags);

    if options.compile_flags.mode == CompileMode::Intermediary {
        for album_root in albums {
            let (man, _) = manifest::build(
                &album_root,
                &project_root,
                &config_json,
                &gen_cfg,
                &active_flags,
                options.compile_flags.no_extensions,
            )?;
            if options.compile_flags.pretty {
                println!("{}", serde_json::to_string_pretty(&man)?);
            } else {
                println!("{}", serde_json::to_string(&man)?);
            }
        }
        return Ok(());
    }

    let registry = config_json.get("compiler_registry").and_then(Value::as_object);
    let has_extensions = registry.is_some_and(|r| {
        r.values().any(|v| v.get("provider").and_then(Value::as_str) == Some("extension"))
    });

    let effective_no_extensions = options.compile_flags.no_extensions || !has_extensions;

    let stream_ctx = StreamContext {
        albums: albums.clone(),
        config: Arc::new(config_json.clone()),
        project_root: Arc::new(project_root.clone()),
        gen_cfg: Arc::new(gen_cfg),
        active_flags,
        target: options.compile_flags.target,
        jobs: options.jobs,
        no_extensions: effective_no_extensions,
        notify_tx: options.notify_tx,
    };

    if effective_no_extensions {
        log::info!("Compiling {} albums (Native Only)...", albums.len());
        return stream::run(None, stream_ctx).await;
    }

    let home = dirs::home_dir().context("No home dir")?;

    let explicit_flake =
        config.extensions.as_ref().map(|ext| PathBuf::from(&ext.folder).join(&ext.flake));

    let mut nix_env = get_nix_env(&project_root, explicit_flake)?;
    nix_env.insert("HOME".to_string(), home.to_string_lossy().to_string());

    log::info!("Compiling {} albums...", albums.len());

    let child =
        kernel::spawn(&serde_json::from_value(config_json.clone())?, &project_root, &nix_env)?;

    stream::run(Some(child), stream_ctx).await
}
