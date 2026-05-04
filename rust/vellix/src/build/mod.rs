use anyhow::{Context, Result};
use sha1::{Sha1, Digest};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use vellum::config::AppConfig;
use vellum::utils::expand_path;

pub fn verify_seedability(torrent_path: &Path, source_disk: &Path) -> Result<()> {
    let torrent = lava_torrent::torrent::v1::Torrent::read_from_file(torrent_path)
        .map_err(|e| anyhow::anyhow!("Failed to parse torrent: {e}"))?;
        
    let piece_length = torrent.piece_length as usize;
    let pieces = &torrent.pieces;
    let mut piece_idx = 0;
    let mut hasher = Sha1::new();
    let mut bytes_in_piece = 0;

    let mut paths_and_sizes = Vec::new();
    if let Some(files) = &torrent.files {
        for f in files {
            let mut p = source_disk.to_path_buf();
            for segment in &f.path {
                p.push(segment);
            }
            paths_and_sizes.push((p, f.length as u64));
        }
    } else {
        paths_and_sizes.push((source_disk.to_path_buf(), torrent.length as u64));
    }

    for (path, size) in paths_and_sizes {
        if !path.exists() {
            anyhow::bail!("Missing file: {}", path.display());
        }
        if std::fs::metadata(&path)?.len() != size {
            anyhow::bail!("Size mismatch for {}. Expected {size}, found {}", path.display(), std::fs::metadata(&path)?.len());
        }
        
        let mut file = std::fs::File::open(&path)?;
        let mut buffer = vec![0u8; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 { break; }
            
            let mut slice = &buffer[..n];
            while !slice.is_empty() {
                let needed = piece_length - bytes_in_piece;
                let take = std::cmp::min(slice.len(), needed);
                hasher.update(&slice[..take]);
                bytes_in_piece += take;
                slice = &slice[take..];
                
                if bytes_in_piece == piece_length {
                    let hash = hasher.finalize_reset();
                    if piece_idx >= pieces.len() || hash.as_slice() != pieces[piece_idx] {
                        anyhow::bail!("Hash mismatch at piece {piece_idx}");
                    }
                    piece_idx += 1;
                    bytes_in_piece = 0;
                }
            }
        }
    }
    
    if bytes_in_piece > 0 {
        let hash = hasher.finalize();
        if piece_idx >= pieces.len() || hash.as_slice() != pieces[piece_idx] {
            anyhow::bail!("Final piece failed hash check");
        }
    }
    
    Ok(())
}

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

    build_album(&target_path, &store_path, &flake_uri)?;

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
) -> Result<()> {
    let album_info = crate::get::parse_album_nix(target_path)?;
    let target = target_path.parent().unwrap();
    
    let torrent_path = target.join(album_info.torrent_file.trim_start_matches("./"));
    let source_disk = target.join(album_info.source_disk.trim_start_matches("./"));

    log::info!("Verifying seedability (Black Box Check)...");
    verify_seedability(&torrent_path, &source_disk)?;
    log::info!("Seeding check passed! Files perfectly match torrent.");

    let album_id = calculate_hash(&target.to_string_lossy());
    let gc_roots_dir = store_path.join("gcroots").join("albums");
    fs::create_dir_all(&gc_roots_dir).context("Failed to create gcroots directory")?;

    let result_link = gc_roots_dir.join(&album_id);
    let expr = format!("(import ./album.nix {{ vellix = (builtins.getFlake \"{flake_uri}\").lib; }})");

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
