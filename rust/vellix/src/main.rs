mod build;
mod get;
mod manifest;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Manifest {
        #[arg(long)]
        torrent: String,
        #[arg(long, default_value = "flac,wav")]
        tracks: String,
    },
    Get {
        #[arg(value_name = "PATH")]
        path: Option<String>,
    },
    Build {
        #[arg(value_name = "PATH")]
        path: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Manifest { torrent, tracks } => {
            manifest::run(&torrent, &tracks)?;
        }
        Commands::Get { path } => {
            get::run(path)?;
        }
        Commands::Build { path } => {
            build::run(path)?;
        }
    }

    Ok(())
}
