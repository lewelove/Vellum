#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

mod compile;
mod config;
mod harvest;
mod manifest;
mod run;
mod server;
mod update;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Harvest {
        #[arg(value_name = "PATHS", required = true, num_args = 1..)]
        paths: Vec<String>,
        #[arg(long)]
        pretty: bool,
        #[arg(long, short = 'j')]
        jobs: Option<usize>,
    },
    Server {
        #[arg(long, default_value = "8000")]
        port: u16,
    },
    Compile {
        #[arg(value_name = "PATH", required = true)]
        path: String,
        #[arg(long)]
        stdout: bool,
        #[arg(long)]
        intermediary: bool,
        #[arg(long)]
        pretty: bool,
        #[arg(long, value_delimiter = ',')]
        flags: Vec<String>,
    },
    Update {
        #[arg(value_name = "PATH")]
        path: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long, short = 'j')]
        jobs: Option<usize>,
    },
    Manifest {
        #[arg(long)]
        force: bool,
    },
    Run {
        #[arg(value_name = "COMMAND", required = true)]
        script_cmd: String,
        #[arg(value_name = "PATH")]
        path: Option<String>,
        #[arg(long)]
        playing: bool,
    },
}

#[must_use]
pub fn expand_path(path_str: &str) -> PathBuf {
    if path_str.starts_with('~')
        && let Some(home) = dirs::home_dir()
    {
        if path_str == "~" {
            return home;
        }
        if let Some(stripped) = path_str.strip_prefix("~/") {
            return home.join(stripped);
        }
    }
    PathBuf::from(path_str)
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("mpd_protocol", log::LevelFilter::Warn)
        .with_module_level("mpd_client", log::LevelFilter::Warn)
        .with_module_level("tracing", log::LevelFilter::Warn)
        .env()
        .init()
        .ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Harvest {
            paths,
            pretty,
            jobs,
        } => {
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

            harvest::run(targets, pretty);
            Ok(())
        }
        Commands::Server { port } => server::run(port).await,
        Commands::Compile {
            path,
            stdout,
            intermediary,
            pretty,
            flags,
        } => {
            let expanded = expand_path(&path);
            let options = compile::CompileOptions {
                target_path: expanded,
                flags,
                specific_albums: None,
                jobs: None,
                notify_tx: None,
                compile_flags: compile::CompileFlags {
                    mode: if intermediary {
                        compile::CompileMode::Intermediary
                    } else {
                        compile::CompileMode::Standard
                    },
                    target: if stdout {
                        compile::ExportTarget::Stdout
                    } else {
                        compile::ExportTarget::File
                    },
                    pretty,
                },
            };
            compile::run(options).await
        }
        Commands::Update {
            path,
            force,
            jobs,
        } => {
            let expanded = path.map(|p| expand_path(&p));
            update::run(expanded, force, jobs).await
        }
        Commands::Manifest { force } => manifest::run(force).await,
        Commands::Run { script_cmd, path, playing } => run::execute(script_cmd, path, playing).await,
    }
}
