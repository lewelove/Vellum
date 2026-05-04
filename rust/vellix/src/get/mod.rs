use anyhow::{Context, Result};
use std::path::Path;
use vellum::config::AppConfig;
use vellum::utils::expand_path;

#[derive(Debug)]
pub struct AlbumInfo {
    pub source_disk: String,
    pub torrent_file: String,
    pub torrent_sha256: String,
}

pub fn parse_album_nix(path: &Path) -> Result<AlbumInfo> {
    let content = std::fs::read_to_string(path)?;
    let re_disk = regex::Regex::new(r"sourceDisk\s*=\s*([^;]+)").unwrap();
    let re_torrent = regex::Regex::new(r"(?s)sourceTorrent\s*=\s*\{\s*file\s*=\s*([^;]+);\s*sha256\s*=\s*([^;]+);\s*\}").unwrap();

    let source_disk = re_disk.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
    
    let t_caps = re_torrent.captures(&content);
    let torrent_file = t_caps.as_ref().and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
    let torrent_sha256 = t_caps.as_ref().and_then(|c| c.get(2)).map(|m| m.as_str().trim().trim_matches('"').to_string()).unwrap_or_default();

    Ok(AlbumInfo { source_disk, torrent_file, torrent_sha256 })
}

pub fn run(album_path: Option<String>) -> Result<()> {
    let (config, _, _) = AppConfig::load().context("Failed to load config")?;
    let nix_config = config.nix.as_ref().context("Missing [nix] configuration")?;
    let get_cfg = nix_config.get.as_ref().context("Missing [nix.get] configuration")?;
    let cmds = get_cfg.commands.as_ref().context("Missing [nix.get.commands] configuration")?;

    let target_path = if let Some(a) = album_path {
        let p = expand_path(&a).canonicalize().context("Album path does not exist")?;
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

    let album_info = parse_album_nix(&target_path)?;
    let album_dir = target_path.parent().unwrap();

    let torrent_path = album_dir.join(album_info.torrent_file.trim_start_matches("./"));
    if !torrent_path.exists() {
        anyhow::bail!("Torrent file not found: {}", torrent_path.display());
    }

    let output = std::process::Command::new("nix")
        .args(["hash", "file", torrent_path.to_str().unwrap()])
        .output()?;
    let current_sha256 = String::from_utf8(output.stdout)?.trim().to_string();

    if current_sha256 != album_info.torrent_sha256 {
        anyhow::bail!("Torrent file hash mismatch! Expected {}, got {}", album_info.torrent_sha256, current_sha256);
    }

    let cmd_tpl = cmds.get("torrent").context("No 'torrent' command configured")?;
    let mut final_cmd = cmd_tpl.clone();
    
    let shell_quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));

    final_cmd = final_cmd.replace("${sourceDisk}", &shell_quote(&album_dir.to_string_lossy()));
    final_cmd = final_cmd.replace("${sourceTorrent}", &shell_quote(&torrent_path.to_string_lossy()));

    log::info!("Executing: {final_cmd}");
    let status = std::process::Command::new("sh").arg("-c").arg(&final_cmd).status()?;

    if status.success() {
        log::info!("Download initiated to {}", album_dir.display());
    } else {
        log::error!("Command failed with status: {status}");
    }

    Ok(())
}
