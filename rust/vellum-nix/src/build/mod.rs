use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::process::Command;
use vellum::config::AppConfig;
use vellum::utils::expand_path;

pub fn run(library: bool, album_path: Option<String>) -> Result<()> {
    let (config, _, _) = AppConfig::load().context("Failed to load config")?;
    let nix_config = config.nix.as_ref().context("Missing [nix] configuration in config.toml")?;
    let store_path = expand_path(&nix_config.store)
        .canonicalize()
        .context("Custom nix store path does not exist or is inaccessible")?;

    let mut flake_uri = nix_config.flake.clone();
    if flake_uri.starts_with('/') || flake_uri.starts_with('~') {
        let expanded = expand_path(&flake_uri);
        let canon = expanded.canonicalize().context("Could not find flake path")?;
        
        let flake_dir = if canon.is_file() {
            canon.parent().context("Flake path has no parent")?.to_path_buf()
        } else {
            canon
        };
        
        flake_uri = format!("path:{}", flake_dir.display());
    }

    let mut targets = Vec::new();

    if library {
        let lib_root = expand_path(&config.storage.library_root);
        let scan_depth = config
            .compiler
            .as_ref()
            .and_then(|c| c.scan_depth)
            .unwrap_or(4);
        let dirs = vellum::scanner::find_target_albums(&lib_root, scan_depth)?;
        for dir in dirs {
            if dir.join("album.nix").exists() {
                targets.push(dir);
            }
        }
    } else if let Some(a) = album_path {
        let p = expand_path(&a)
            .canonicalize()
            .context("Album path does not exist")?;
        if p.is_dir() && p.join("album.nix").exists() {
            targets.push(p);
        } else if p.is_file() && p.file_name().unwrap_or_default() == "album.nix" {
            targets.push(p.parent().unwrap().to_path_buf());
        }
    }

    for target in targets {
        build_album(&target, &store_path, &flake_uri, nix_config.stage.as_deref())?;
    }

    Ok(())
}

fn calculate_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn build_album(
    target: &Path,
    store_path: &Path,
    flake_uri: &str,
    stage_path_str: Option<&str>,
) -> Result<()> {
    let album_id = calculate_hash(&target.to_string_lossy());
    let gc_roots_dir = store_path.join("gcroots").join("albums");
    fs::create_dir_all(&gc_roots_dir).context("Failed to create gcroots directory")?;

    let result_link = gc_roots_dir.join(&album_id);

    let album_info = crate::get::parse_album_nix(&target.join("album.nix"))?;

    let mut staging_src_env = None;
    if album_info.source_disk.is_none() {
        if let Some(stage) = stage_path_str {
            let stage_dir = expand_path(stage);
            let staging_path = crate::get::calculate_staging_path(&album_info, &stage_dir, target)?;
            
            if staging_path.exists() {
                staging_src_env = Some(staging_path.to_string_lossy().to_string());
            } else {
                anyhow::bail!("Source not found in stage: {}. Please run `vellum-nix get` for this album.", staging_path.display());
            }
        } else {
            anyhow::bail!("No sourceDisk found in album.nix and [nix] stage is not configured.");
        }
    }

    log::info!("Evaluating nix expression for: {}", target.display());

    let expr = format!("(import ./album.nix {{ vellum = (builtins.getFlake \"{flake_uri}\").lib; }})");

    let mut cmd = Command::new("nix");
    cmd.arg("build")
        .arg("--store")
        .arg(store_path)
        .arg("--impure")
        .arg("--expr")
        .arg(&expr)
        .arg("--out-link")
        .arg(&result_link)
        .current_dir(target);

    if let Some(staging_src) = staging_src_env {
        cmd.env("VELLUM_STAGING_SRC", staging_src);
    }

    let status = cmd.status().context("Failed to execute nix build binary")?;

    if !status.success() {
        anyhow::bail!(
            "Nix build failed with exit code {} for {}",
            status.code().unwrap_or(-1),
            target.display()
        );
    }

    let logical_path = fs::read_link(&result_link).with_context(|| {
        format!(
            "Nix build claimed success but result link {} was not found",
            result_link.display()
        )
    })?;

    let stripped_path = logical_path.strip_prefix("/").unwrap_or(&logical_path);
    let physical_store_path = store_path.join(stripped_path);

    materialize_output(&physical_store_path, target, store_path)?;

    Ok(())
}

fn materialize_output(store_dir: &Path, target_dir: &Path, store_path: &Path) -> Result<()> {
    let entries = fs::read_dir(store_dir).with_context(|| {
        format!(
            "Could not read nix store directory: {}",
            store_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_type = entry.file_type()?;

        if file_name == "album.nix" {
            continue;
        }

        let mut store_file = entry.path();
        let target_file = target_dir.join(&file_name);

        if let Ok(meta) = fs::symlink_metadata(&target_file) {
            if meta.is_dir() {
                fs::remove_dir_all(&target_file)?;
            } else {
                fs::remove_file(&target_file)?;
            }
        }

        if file_type.is_symlink() {
            let resolved_path = fs::read_link(&store_file)?;
            if resolved_path.starts_with("/nix/store") {
                let stripped = resolved_path.strip_prefix("/").unwrap();
                store_file = store_path.join(stripped);
            } else if resolved_path.is_relative() {
                store_file = store_dir.join(resolved_path);
            } else {
                store_file = resolved_path;
            }
        }

        if fs::hard_link(&store_file, &target_file).is_err() {
            std::os::unix::fs::symlink(&store_file, &target_file).with_context(|| {
                format!(
                    "Failed to create link (hard or sym) for {} at {}",
                    store_file.display(),
                    target_file.display()
                )
            })?;
        }
    }
    Ok(())
}
