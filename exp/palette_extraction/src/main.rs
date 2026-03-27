mod kmeans;
mod kmeans_msc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Kmeans {
        path: String,
        #[arg(short, long)]
        args: Option<String>,
    },
    KmeansMsc {
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Kmeans { path, args } => {
            kmeans::run_pure_kmeans(path, args.as_deref().unwrap_or(""));
        }
        Commands::KmeansMsc { path } => {
            kmeans_msc::run_hybrid_msc(path);
        }
    }
}
