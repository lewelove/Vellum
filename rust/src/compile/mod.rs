pub mod builder;
pub mod engine;
pub mod resolvers;

use crate::config::AppConfig;
use anyhow::{Context, Result};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

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
}

pub struct CompileOptions {
    pub target_path: PathBuf,
    pub flags: Vec<String>,
    pub specific_albums: Option<Vec<PathBuf>>,
    pub jobs: Option<usize>,
    pub notify_tx: Option<mpsc::Sender<engine::stream::AlbumUpdateSignal>>,
    pub compile_flags: CompileFlags,
}

pub async fn run(mut options: CompileOptions) -> Result<()> {
    let (config, _raw, path) = AppConfig::load().context("Config failed")?;
    let project_root = path.parent().unwrap().to_path_buf();
    if !options.flags.contains(&"default".to_string()) {
        options.flags.push("default".to_string());
    }

    let albums = if let Some(l) = options.specific_albums {
        l
    } else {
        let scan_depth = config.compiler.as_ref().and_then(|c| c.scan_depth).unwrap_or(4);
        builder::scan::find_target_albums(&options.target_path, scan_depth)
    };

    if albums.is_empty() {
        return Ok(());
    }
    let config_json = serde_json::to_value(&config)?;
    let manifest_cfg = config_json
        .get("manifest")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let active_flags = Arc::new(options.flags);

    if options.compile_flags.mode == CompileMode::Intermediary {
        for root in albums {
            let m = builder::build(
                &root,
                &project_root,
                &config_json,
                &manifest_cfg,
                &active_flags,
            )?;
            if options.compile_flags.pretty {
                println!("{}", serde_json::to_string_pretty(&m)?);
            } else {
                println!("{}", serde_json::to_string(&m)?);
            }
        }
        return Ok(());
    }

    let ctx = engine::stream::StreamContext {
        albums: albums.clone(),
        config: Arc::new(config_json.clone()),
        project_root: Arc::new(project_root.clone()),
        manifest_cfg: Arc::new(manifest_cfg),
        active_flags,
        target: options.compile_flags.target,
        jobs: options.jobs,
        notify_tx: options.notify_tx,
    };

    engine::stream::run(ctx).await
}
