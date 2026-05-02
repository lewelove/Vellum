use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;
use vellum::config::AppConfig;
use vellum::expand_path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(long)]
        library: bool,
        #[arg(long, value_name = "PATH")]
        album: Option<String>,
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
        Commands::Build { library, album } => {
            run_build(library, album).await?;
        }
    }

    Ok(())
}

async fn run_build(library: bool, album: Option<String>) -> Result<()> {
    let (config, _, _) = AppConfig::load().context("Failed to load config")?;
    let nix_config = config.nix.as_ref().context("Missing [nix] configuration in config.toml")?;
    let store_path = expand_path(&nix_config.store).canonicalize().context("Custom nix store path does not exist or is inaccessible")?;
    
    let mut flake_uri = nix_config.flake.clone();
    if flake_uri.starts_with('/') || flake_uri.starts_with('~') {
        let expanded = expand_path(&flake_uri);
        let canon = expanded.canonicalize().context("Could not find flake directory")?;
        flake_uri = format!("path:{}", canon.display());
    }

    let mut targets = Vec::new();

    if library {
        let lib_root = expand_path(&config.storage.library_root);
        let scan_depth = config.compiler.as_ref().and_then(|c| c.scan_depth).unwrap_or(4);
        let dirs = vellum::compile::builder::scan::find_target_albums(&lib_root, scan_depth)?;
        for dir in dirs {
            if dir.join("album.nix").exists() {
                targets.push(dir);
            }
        }
    } else if let Some(a) = album {
        let p = expand_path(&a).canonicalize().context("Album path does not exist")?;
        if p.is_dir() && p.join("album.nix").exists() {
            targets.push(p);
        } else if p.is_file() && p.file_name().unwrap_or_default() == "album.nix" {
            targets.push(p.parent().unwrap().to_path_buf());
        }
    }

    for target in targets {
        build_album(&target, &store_path, &flake_uri)?;
    }

    Ok(())
}

fn build_album(target: &Path, store_path: &Path, flake_uri: &str) -> Result<()> {
    let result_link = target.join(".vellum-result");

    if result_link.exists() {
        fs::remove_file(&result_link).ok();
    }

    log::info!("Evaluating nix expression for: {}", target.display());

    let expr = format!(
        "(import ./album.nix {{ vellum = (builtins.getFlake \"{}\").lib; }})",
        flake_uri
    );

    let status = Command::new("nix")
        .arg("build")
        .arg("--store")
        .arg(store_path)
        .arg("--impure")
        .arg("--expr")
        .arg(&expr)
        .arg("--out-link")
        .arg(&result_link)
        .current_dir(target)
        .status()
        .context("Failed to execute nix build binary")?;

    if !status.success() {
        anyhow::bail!("Nix build failed with exit code {} for {}", status.code().unwrap_or(-1), target.display());
    }

    let logical_path = fs::read_link(&result_link)
        .with_context(|| format!("Nix build claimed success but result link {} was not found", result_link.display()))?;

    let stripped_path = logical_path.strip_prefix("/").unwrap_or(&logical_path);
    let physical_store_path = store_path.join(stripped_path);

    materialize_output(&physical_store_path, target)?;
    
    fs::remove_file(&result_link).ok();

    Ok(())
}

fn materialize_output(store_dir: &Path, target_dir: &Path) -> Result<()> {
    let entries = fs::read_dir(store_dir)
        .with_context(|| format!("Could not read nix store directory: {}", store_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();

        if file_name == "album.nix" {
            continue;
        }

        let store_file = entry.path();
        let target_file = target_dir.join(&file_name);

        if target_file.exists() {
            if target_file.is_dir() {
                fs::remove_dir_all(&target_file)?;
            } else {
                fs::remove_file(&target_file)?;
            }
        }

        if store_file.is_file() {
            if fs::hard_link(&store_file, &target_file).is_err() {
                std::os::unix::fs::symlink(&store_file, &target_file)
                    .with_context(|| format!("Failed to link {} to {}", store_file.display(), target_file.display()))?;
            }
        } else if store_file.is_dir() {
            std::os::unix::fs::symlink(&store_file, &target_file)
                .with_context(|| format!("Failed to symlink directory {} to {}", store_file.display(), target_file.display()))?;
        }
    }
    Ok(())
}
