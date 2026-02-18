mod harvest;
mod server;
mod config;
mod compile;

use clap::{Parser, Subcommand};
use std::path::{PathBuf};
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scans directories and outputs one JSON object per file to stdout
    Harvest {
        #[arg(value_name = "PATHS", required = true, num_args = 1..)]
        paths: Vec<String>,

        /// Output human-readable indented JSON
        #[arg(long)]
        pretty: bool,

        /// Number of threads to use [default: all cores]
        #[arg(long, short = 'j')]
        jobs: Option<usize>,
    },
    /// Starts the Vellum Server (Rust implementation)
    Server {
        /// Port to listen on
        #[arg(long, default_value = "8000")]
        port: u16,
    },
    /// Compiles metadata.toml into metadata.lock.json using Bun resolvers
    Compile {
        #[arg(value_name = "PATH", required = true)]
        path: String,
    }
}

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

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Harvest { paths, pretty, jobs } => {
            if let Some(j) = jobs {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(j)
                    .build_global()
                    .context("Failed to build thread pool")?;
            }

            let mut targets = Vec::new();
            for p in paths {
                let expanded = expand_path(&p);
                if let Ok(canon) = expanded.canonicalize() {
                    targets.push(canon);
                } else {
                    targets.push(expanded);
                }
            }
                
            harvest::run(targets, pretty)
        },
        Commands::Server { port } => {
            server::run(port).await
        },
        Commands::Compile { path } => {
            let expanded = expand_path(&path);
            compile::run(expanded)
        }
    }
}
