mod build;
mod get;

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
    Build {
        #[arg(value_name = "PATH")]
        path: Option<String>,
        #[arg(long)]
        library: bool,
    },
    Get {
        #[arg(value_name = "PATH")]
        path: Option<String>,
        #[arg(long)]
        torrent: bool,
        #[arg(long)]
        url: bool,
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
        Commands::Build { path, library } => {
            build::run(library, path)?;
        }
        Commands::Get { path, torrent, url } => {
            get::run(torrent, url, path)?;
        }
    }

    Ok(())
}
