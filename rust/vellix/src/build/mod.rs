use anyhow::{Context, Result};
use lava_torrent::torrent::v1::Torrent;
use sha2::Digest;
use std::fs;
use std::path::Path;
use std::process::Command;
use vellum::config::AppConfig;
use vellum::utils::expand_path;

pub fn run(album_path: Option<String>) -> Result<()> {
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

    let target_path = if let Some(a) = album_path {
        let p = expand_path(&a)
            .canonicalize()
            .context("Album path does not exist")?;
        if p.is_dir() && p.join("album.nix").exists() {
            p.join("album.nix")
        } else if p.is_file() && p.file_name().unwrap_or_default() == "album.nix" {
            p
        } else {
            anyhow::bail!("No album.nix found at specified path");
        }
    } else {
        let curr = std::env::current_dir()?;
        if curr.join("album.nix").exists() {
            curr.join("album.nix")
        } else {
            anyhow::bail!("No album.nix found in current directory");
        }
    };

    build_album(&target_path, &store_path, &flake_uri, &config)?;

    Ok(())
}

fn calculate_hash(data: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn build_album(
    target_path: &Path,
    store_path: &Path,
    flake_uri: &str,
    config: &AppConfig,
) -> Result<()> {
    let target = target_path.parent().unwrap();
    let album_info = crate::get::parse_album_nix(target_path)?;

    let torrent_file_path = if album_info.torrent_file.starts_with("./") {
        target.join(album_info.torrent_file.trim_start_matches("./"))
    } else {
        target.join(&album_info.torrent_file)
    };
    
    let source_disk = crate::get::resolve_source_disk(&album_info, target, config);

    let torrent_name = Torrent::read_from_file(&torrent_file_path)
        .map_or_else(|_| album_info.pname.clone(), |t| t.name);

    let mut vars = std::collections::HashMap::new();
    vars.insert("sourceDisk".to_string(), source_disk.to_string_lossy().to_string());
    vars.insert("sourceTorrent.file".to_string(), torrent_file_path.to_string_lossy().to_string());
    vars.insert("sourceTorrent.sha256".to_string(), album_info.torrent_sha256);
    vars.insert("sourceTorrent.name".to_string(), torrent_name);

    if let Some(nix_cfg) = &config.nix
        && let Some(build_cfg) = &nix_cfg.build
        && let Some(cmds) = &build_cfg.commands
        && let Some(verify_cmd_tpl) = cmds.get("verify_torrent")
    {
        let final_cmd = crate::get::resolve_template(verify_cmd_tpl, &vars);
        log::info!("Executing verification: {final_cmd}");
        
        let status = Command::new("sh")
            .arg("-c")
            .arg(&final_cmd)
            .current_dir(target)
            .status()?;
            
        if !status.success() {
            anyhow::bail!("Verification command failed with status: {status}");
        }
        log::info!("Seeding check passed!");
    }

    let album_id = calculate_hash(&target.to_string_lossy());
    let gc_roots_dir = store_path.join("gcroots").join("albums");
    fs::create_dir_all(&gc_roots_dir).context("Failed to create gcroots directory")?;

    let result_link = gc_roots_dir.join(&album_id);
    let expr = format!("(import ./album.nix {{ vellix = (builtins.getFlake \"{flake_uri}\").lib; }})");

    let mut cmd = Command::new("nix");
    cmd.env("VELLUM_STAGING_SRC", &source_disk);
    cmd.arg("build")
        .arg("--store")
        .arg(store_path)
        .arg("--impure")
        .arg("--expr")
        .arg(&expr)
        .arg("--out-link")
        .arg(&result_link)
        .current_dir(target);

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
