use crate::config::AppConfig;
use crate::expand_path;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub async fn run(action: String, library: bool, album: Option<String>) -> Result<()> {
    if action != "build" {
        anyhow::bail!("Unsupported nix action");
    }

    let (config, _, _) = AppConfig::load().context("Failed to load config")?;
    let nix_config = config.nix.context("Missing [nix] configuration in config.toml")?;
    let store_path = expand_path(&nix_config.store);
    let flake_uri = nix_config.flake;

    let mut targets = Vec::new();

    if library {
        let lib_root = expand_path(&config.storage.library_root);
        let scan_depth = config
            .compiler
            .as_ref()
            .and_then(|c| c.scan_depth)
            .unwrap_or(4);
        let dirs = crate::compile::builder::scan::find_target_albums(&lib_root, scan_depth)?;
        for dir in dirs {
            if dir.join("album.nix").exists() {
                targets.push(dir);
            }
        }
    } else if let Some(a) = album {
        let p = expand_path(&a).canonicalize()?;
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

    log::info!("Evaluating nix expression for: {}", target.display());

    let expr = format!("(import ./album.nix {{ vellum = (builtins.getFlake \"{}\").lib; }})", flake_uri);

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
        .status()?;

    if !status.success() {
        anyhow::bail!("Nix build failed for {}", target.display());
    }

    if let Ok(real_path) = fs::read_link(&result_link) {
        materialize_output(&real_path, target)?;
        fs::remove_file(&result_link)?;
    }

    Ok(())
}

fn materialize_output(store_dir: &Path, target_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(store_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if file_name == "album.nix" {
            continue;
        }

        let store_file = entry.path();
        let target_file = target_dir.join(&file_name);

        if target_file.exists() {
            if target_file.is_file() || target_file.is_symlink() {
                fs::remove_file(&target_file)?;
            } else if target_file.is_dir() {
                fs::remove_dir_all(&target_file)?;
            }
        }

        if store_file.is_file() {
            if fs::hard_link(&store_file, &target_file).is_err() {
                std::os::unix::fs::symlink(&store_file, &target_file)?;
            }
        } else if store_file.is_dir() {
            std::os::unix::fs::symlink(&store_file, &target_file)?;
        }
    }
    Ok(())
}
