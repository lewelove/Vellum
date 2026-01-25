mod config;
mod generate;

use clap::Parser;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use config::AppConfig;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[arg(value_name = "PATH")]
    path: Option<String>,

    #[arg(long, short)]
    force: bool,

    #[arg(long, short = 'j')]
    jobs: Option<usize>,
}

fn load_config() -> Result<AppConfig> {
    let candidates = vec![
        Path::new("config.toml"),
        Path::new("../config.toml"),
    ];
    for path in candidates {
        if path.exists() {
            println!("Loading config from: {:?}", path.canonicalize()?);
            let content = fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            return Ok(config);
        }
    }
    anyhow::bail!("config.toml not found.")
}

fn resolve_library_root(config: &AppConfig, cli_path: Option<String>) -> Result<PathBuf> {
    if let Some(p) = cli_path {
        let path = PathBuf::from(p);
        if !path.exists() {
            anyhow::bail!("Provided path does not exist: {:?}", path);
        }
        return Ok(path.canonicalize()?);
    }
    let lib_root = &config.storage.library_root;
    let path = if lib_root.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            home.join(lib_root.trim_start_matches("~/"))
        } else {
            PathBuf::from(lib_root)
        }
    } else {
        PathBuf::from(lib_root)
    };
    if !path.exists() {
        anyhow::bail!("Configured library_root does not exist: {:?}", path);
    }
    Ok(path.canonicalize()?)
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .context("Failed to build thread pool")?;
    }

    let config = load_config()?;
    let target_root = resolve_library_root(&config, args.path)?;
    
    println!("Target Root: {:?}", target_root);

    generate::run(target_root, &config, args.force)
}
