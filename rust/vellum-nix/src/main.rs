use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use vellum::config::AppConfig;
use vellum::expand_path;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Realize {
        #[arg(long)]
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .ok();

    let (config, _, _) = AppConfig::load().context("Failed to load Vellum configuration")?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Realize { path } => {
            let target = expand_path(&path);
            log::info!("Realizing album at {}", target.display());
        }
    }

    Ok(())
}
