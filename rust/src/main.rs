#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

mod compile;
mod config;
mod egui;
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
    #[command(name = "egui")]
    Egui,
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
        Commands::Egui => {
            let (config, _, _) = config::AppConfig::load().context("Failed to load application configuration")?;
            let library_root = expand_path(&config.storage.library_root).canonicalize()?;
            
            let server_config = server::state::AppConfig {
                library_root: library_root.clone(),
                thumbnail_root: config.storage.thumbnail_cache_folder.map(|p| expand_path(&p)),
                thumbnail_size: config.theme.as_ref().map_or(200, |t| t.thumbnail_size),
                shader: None,
                resolved_shader_path: None,
                resolved_css_path: None,
                resolved_facets_path: None,
                resolved_sorters_path: None,
                resolved_shelves_path: None,
            };
            
            let mut library = server::library::Library::new(library_root);
            library.scan();
            let library_arc = std::sync::Arc::new(tokio::sync::RwLock::new(library));
            let (tx, _) = tokio::sync::broadcast::channel(100);
            
            let mpd_engine = server::mpd::start_actor(
                tx.clone(),
                std::sync::Arc::clone(&library_arc),
                std::sync::Arc::new(server_config.clone()),
            );
            
            let app_state = std::sync::Arc::new(server::state::AppState {
                library: library_arc,
                ui_state: tokio::sync::RwLock::new(serde_json::json!({})),
                tx,
                config: tokio::sync::RwLock::new(server_config),
                mpd_engine,
            });
            
            egui::run(app_state)?;
            Ok(())
        }
    }
}
