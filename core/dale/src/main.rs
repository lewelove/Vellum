mod compile;
mod harvest;
mod interface;
mod manifest;
mod query;
mod server;
mod update;
mod x;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use libdale::utils::expand_path;

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
    Interface {
        #[arg(value_name = "NAME")]
        name: Option<String>,
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
        #[arg(long)]
        silent: bool,
    },
    Manifest {
        #[arg(value_name = "PATH")]
        path: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long, required_unless_present = "library", conflicts_with = "library")]
        album: bool,
        #[arg(long, required_unless_present = "album", conflicts_with = "album")]
        library: bool,
        #[arg(long)]
        stdout: bool,
    },
    X {
        #[arg(value_name = "NAME", required = true)]
        name: String,
        #[arg(long, short = 'p', group = "target")]
        playing: bool,
        #[arg(long, group = "target")]
        id: Option<String>,
        #[arg(long, short = 'q', group = "target")]
        query: Option<String>,
        #[arg(long, short = 'd', group = "target")]
        directory: Option<String>,
        #[arg(long, short = 'r', group = "target")]
        recursive: Option<String>,
        #[arg(long, short = 'l', group = "target")]
        library: bool,
        #[arg(long)]
        debug: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Query {
        #[arg(value_name = "QUERY")]
        query_str: Option<String>,
        #[arg(long)]
        playing: bool,
        #[arg(long)]
        lock: bool,
        #[arg(long)]
        id: bool,
        #[arg(long)]
        json: bool,
    },
}

fn handle_harvest(paths: Vec<String>, pretty: bool, jobs: Option<usize>) -> Result<()> {
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

async fn handle_compile(
    path: String,
    stdout: bool,
    intermediary: bool,
    pretty: bool,
    flags: Vec<String>,
) -> Result<()> {
    let expanded = expand_path(&path);
    let options = compile::CompileOptions {
        target_path: expanded,
        flags,
        specific_albums: None,
        jobs: None,
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
        ingest_tx: None,
        active_writes: None,
        silent: false,
    };
    let _ = compile::run(options).await?;
    Ok(())
}

fn handle_manifest(
    path: Option<String>,
    force: bool,
    album: bool,
    stdout: bool,
) -> Result<()> {
    let expanded = path.map(|p| expand_path(&p));
    let mode = if album {
        manifest::ManifestMode::Album
    } else {
        manifest::ManifestMode::Library
    };
    let options = manifest::ManifestOptions {
        mode,
        force,
        stdout,
    };
    manifest::run(expanded.as_deref(), &options)
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
        } => handle_harvest(paths, pretty, jobs),
        Commands::Server { port } => server::run(port).await,
        Commands::Interface { name } => interface::execute(name).await,
        Commands::Compile {
            path,
            stdout,
            intermediary,
            pretty,
            flags,
        } => handle_compile(path, stdout, intermediary, pretty, flags).await,
        Commands::Update {
            path,
            force,
            jobs,
            silent,
        } => {
            let expanded = path.map(|p| expand_path(&p));
            update::run(expanded, force, jobs, silent).await
        }
        Commands::Manifest {
            force,
            path,
            album,
            library: _,
            stdout,
        } => handle_manifest(path, force, album, stdout),
        Commands::X {
            name,
            playing,
            id,
            query,
            directory,
            recursive,
            library,
            debug,
            args,
        } => {
            let target = x::TargetFlags {
                playing,
                id,
                query,
                directory,
                recursive,
                library,
            };
            x::execute(name, target, debug, args).await
        }
        Commands::Query {
            query_str,
            playing,
            lock,
            id,
            json,
        } => {
            let flags = query::QueryFlags {
                playing,
                lock,
                id,
                json,
            };
            query::run(query_str, flags).await
        }
    }
}
