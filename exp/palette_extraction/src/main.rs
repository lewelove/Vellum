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
    /// Pure K-Means at k=32 against 256p Nearest source
    Kmeans {
        path: String,
    },
    /// Hybrid: K-Means Compression -> Global Weighted MSC Merge
    KmeansMsc {
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Kmeans { path } => {
            kmeans::run_pure_kmeans(path);
        }
        Commands::KmeansMsc { path } => {
            kmeans_msc::run_hybrid_msc(path);
        }
    }
}
