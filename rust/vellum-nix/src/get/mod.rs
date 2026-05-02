use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vellum::config::AppConfig;
use vellum::utils::expand_path;

pub struct AlbumInfo {
    pub pname: String,
    pub source_disk: Option<String>,
    pub source_torrent: Option<String>,
    pub source_magnet: Option<String>,
    pub source_url: Option<String>,
}

pub fn parse_album_nix(path: &Path) -> Result<AlbumInfo> {
    let content = fs::read_to_string(path)?;
    
    let extract = |field: &str| -> Option<String> {
        let pattern = format!(r#"{field}\s*=\s*(?:"([^"]+)"|([a-zA-Z0-9_./\-]+))"#);
        let re = Regex::new(&pattern).unwrap();
        re.captures(&content)
            .and_then(|c| c.get(1).or_else(|| c.get(2)))
            .map(|m| m.as_str().to_string())
    };

    let pname = extract("pname").context("pname is required in album.nix")?;
    
    Ok(AlbumInfo {
        pname,
        source_disk: extract("sourceDisk"),
        source_torrent: extract("sourceTorrent"),
        source_magnet: extract("sourceMagnet"),
        source_url: extract("sourceUrl"),
    })
}

pub fn calculate_staging_path(info: &AlbumInfo, stage: &Path, album_dir: &Path) -> Result<PathBuf> {
    let mut hasher = Sha256::new();
    
    if let Some(tor) = &info.source_torrent {
        let tor_path = album_dir.join(tor);
        let content = fs::read(&tor_path).with_context(|| {
            format!("Failed to read torrent file to calculate hash: {}", tor_path.display())
        })?;
        hasher.update(content);
    } else if let Some(mag) = &info.source_magnet {
        hasher.update(mag.as_bytes());
    } else if let Some(url) = &info.source_url {
        hasher.update(url.as_bytes());
    } else {
        anyhow::bail!("No source identifier (torrent, magnet, url) found to calculate staging path.");
    }
    
    let hash = format!("{:x}", hasher.finalize());
    let dir_name = format!("{hash}-{}", info.pname);
    
    Ok(stage.join(dir_name))
}

pub fn run(torrent: bool, url: bool, album_path: Option<String>) -> Result<()> {
    let (config, _, _) = AppConfig::load().context("Failed to load config")?;
    
    let nix_config = config.nix.as_ref().context("Missing [nix] configuration in config.toml")?;
    let stage_dir_str = nix_config.stage.as_ref().context("Missing [nix] stage path configuration")?;
    let stage_dir = expand_path(stage_dir_str);
    
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
    let staging_path = calculate_staging_path(&album_info, &stage_dir, album_dir)?;

    fs::create_dir_all(&staging_path)?;

    let cmd_tpl = if torrent {
        cmds.get("torrent").context("No 'torrent' command configured in [nix.get.commands]")?
    } else if url {
        cmds.get("url").context("No 'url' command configured in [nix.get.commands]")?
    } else {
        anyhow::bail!("Please specify the fetch method using --torrent or --url");
    };

    let mut final_cmd = cmd_tpl.clone();
    
    let shell_quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));

    final_cmd = final_cmd.replace("${sourceDisk}", &shell_quote(&staging_path.to_string_lossy()));
    
    if let Some(tor) = &album_info.source_torrent {
        let absolute_tor = album_dir.join(tor).canonicalize().unwrap_or_else(|_| album_dir.join(tor));
        final_cmd = final_cmd.replace("${sourceTorrent}", &shell_quote(&absolute_tor.to_string_lossy()));
    }
    if let Some(mag) = &album_info.source_magnet {
        final_cmd = final_cmd.replace("${sourceMagnet}", &shell_quote(mag));
    }
    if let Some(url) = &album_info.source_url {
        final_cmd = final_cmd.replace("${sourceUrl}", &shell_quote(url));
    }

    log::info!("Executing: {final_cmd}");

    let status = Command::new("sh")
        .arg("-c")
        .arg(&final_cmd)
        .status()
        .context("Failed to spawn shell for execution")?;

    if status.success() {
        log::info!("Successfully staged to {}", staging_path.display());
    } else {
        log::error!("Staging command failed with status: {status}");
    }

    Ok(())
}
