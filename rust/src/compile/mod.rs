pub mod builder;
pub mod engine;
pub mod resolvers;
pub mod runtime;

use crate::config::AppConfig;
use anyhow::{Context, Result};
use serde_json::{Value, json};
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
    let (config, raw, path) = AppConfig::load().context("Config failed")?;
    let project_root = path.parent().unwrap().to_path_buf();
    if !options.flags.contains(&"default".to_string()) {
        options.flags.push("default".to_string());
    }

    let albums = if let Some(l) = options.specific_albums {
        l
    } else {
        builder::scan::find_target_albums(&options.target_path, 4)
    };

    if albums.is_empty() {
        return Ok(());
    }
    let config_json = serde_json::to_value(&raw)?;
    let gen_cfg = config_json
        .get("generate")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let active_flags = Arc::new(options.flags);

    if options.compile_flags.mode == CompileMode::Intermediary {
        for root in albums {
            let (m, _) = builder::build(
                &root,
                &project_root,
                &config_json,
                &gen_cfg,
                &active_flags,
                options.compile_flags.no_extensions,
            )?;
            if options.compile_flags.pretty {
                println!("{}", serde_json::to_string_pretty(&m)?);
            } else {
                println!("{}", serde_json::to_string(&m)?);
            }
        }
        return Ok(());
    }

    let has_ext = config_json
        .get("compiler_registry")
        .and_then(Value::as_object)
        .is_some_and(|r| {
            r.values()
                .any(|v| v.get("provider").and_then(Value::as_str) == Some("extension"))
        });

    let eff_no_ext = options.compile_flags.no_extensions || !has_ext;

    let ctx = engine::stream::StreamContext {
        albums: albums.clone(),
        config: Arc::new(config_json.clone()),
        project_root: Arc::new(project_root.clone()),
        gen_cfg: Arc::new(gen_cfg),
        active_flags,
        target: options.compile_flags.target,
        jobs: options.jobs,
        no_extensions: eff_no_ext,
        notify_tx: options.notify_tx,
    };

    if eff_no_ext {
        return engine::stream::run(None, ctx).await;
    }

    let flake = config
        .extensions
        .as_ref()
        .map(|ext| PathBuf::from(&ext.folder).join(&ext.flake));
    let mut env = runtime::nix::get_nix_env(&project_root, flake)?;
    env.insert(
        "HOME".to_string(),
        dirs::home_dir().unwrap().to_string_lossy().to_string(),
    );

    let child = runtime::kernel::spawn(
        &serde_json::from_value(config_json.clone())?,
        &project_root,
        &env,
    )?;
    engine::stream::run(Some(child), ctx).await
}
