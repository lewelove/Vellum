mod config;
mod generate;
mod harvest;

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use config::AppConfig;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Legacy generation logic (Rust Scanner -> Toml Writer)
    Generate {
        #[arg(value_name = "PATH")]
        path: Option<String>,

        #[arg(long, short)]
        force: bool,

        #[arg(long, short = 'j')]
        jobs: Option<usize>,
    },
    
    /// New Architecture: Scans a directory and outputs one JSON object per file to stdout
    Harvest {
        #[arg(value_name = "PATH")]
        path: String,

        /// Output human-readable indented JSON
        #[arg(long)]
        pretty: bool,
    }
}

fn load_config() -> Result<AppConfig> {
    let candidates = vec![
        Path::new("config.toml"),
        Path::new("../config.toml"),
    ];
    for path in candidates {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            return Ok(config);
        }
    }
    anyhow::bail!("config.toml not found.")
}

/// Manually expands "~" to the user's home directory
fn expand_path(path_str: &str) -> PathBuf {
    if path_str.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            if path_str == "~" {
                return home;
            }
            if let Some(stripped) = path_str.strip_prefix("~/") {
                return home.join(stripped);
            }
        }
    }
    PathBuf::from(path_str)
}

fn resolve_library_root(config_root: Option<&str>, cli_path: Option<String>) -> Result<PathBuf> {
    if let Some(p) = cli_path {
        let expanded = expand_path(&p);
        if !expanded.exists() {
            anyhow::bail!("Provided path does not exist: {:?}", expanded);
        }
        return Ok(expanded.canonicalize()?);
    }
    
    if let Some(lib_root) = config_root {
        let expanded = expand_path(lib_root);
        
        if !expanded.exists() {
            anyhow::bail!("Configured library_root does not exist: {:?}", expanded);
        }
        return Ok(expanded.canonicalize()?);
    }

    anyhow::bail!("No path provided and no config found.")
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { path, force, jobs } => {
            if let Some(j) = jobs {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(j)
                    .build_global()
                    .context("Failed to build thread pool")?;
            }

            let config = load_config()?;
            let target_root = resolve_library_root(Some(&config.storage.library_root), path)?;
            
            eprintln!("Target Root: {:?}", target_root);
            generate::run(target_root, &config, force)
        },
        
        Commands::Harvest { path, pretty } => {
            let expanded = expand_path(&path);
            let target_root = expanded.canonicalize()
                .with_context(|| format!("Invalid path: {:?}", expanded))?;
                
            harvest::run(target_root, pretty)
        }
    }
}
