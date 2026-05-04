use anyhow::{Context, Result};
use lava_torrent::torrent::v1::Torrent;
use std::path::{Path, PathBuf};
use vellum::config::AppConfig;
use vellum::utils::expand_path;

#[derive(Debug)]
pub struct AlbumInfo {
    pub pname: String,
    pub source_disk: String,
    pub torrent_file: String,
    pub torrent_sha256: String,
}

pub fn extract_nix_attr(content: &str, key_path: &str) -> Option<String> {
    let parts: Vec<&str> = key_path.split('.').collect();
    if parts.len() == 1 {
        let pattern = format!(r"(?s){}\s*=\s*([^;]+);", regex::escape(parts[0]));
        let re = regex::Regex::new(&pattern).unwrap();
        re.captures(content).and_then(|c| c.get(1)).map(|m| m.as_str().trim().trim_matches('"').to_string())
    } else if parts.len() == 2 {
        let pattern = format!(r"(?s){}\s*=\s*\{{[^\}}]*?{}\s*=\s*([^;]+);", regex::escape(parts[0]), regex::escape(parts[1]));
        let re = regex::Regex::new(&pattern).unwrap();
        re.captures(content).and_then(|c| c.get(1)).map(|m| m.as_str().trim().trim_matches('"').to_string())
    } else {
        None
    }
}

pub fn resolve_source_disk(album_info: &AlbumInfo, base_dir: &Path, config: &AppConfig) -> PathBuf {
    if !album_info.source_disk.is_empty() && album_info.source_disk != "." {
        if album_info.source_disk.starts_with("./") {
            base_dir.join(album_info.source_disk.trim_start_matches("./"))
        } else if album_info.source_disk.starts_with('/') {
            PathBuf::from(&album_info.source_disk)
        } else {
            base_dir.join(&album_info.source_disk)
        }
    } else {
        let stage_root = config.nix.as_ref()
            .and_then(|n| n.stage.clone())
            .map_or_else(|| base_dir.join(".vellum/stage"), |s| vellum::utils::expand_path(&s));
        
        let output = std::process::Command::new("nix")
            .args(["hash", "to-base32", &album_info.torrent_sha256])
            .output();
        
        let truncated = if let Ok(out) = output {
            if out.status.success() {
                let nix32 = String::from_utf8(out.stdout).unwrap_or_default().trim().to_string();
                nix32.chars().take(32).collect::<String>()
            } else {
                album_info.torrent_sha256.trim_start_matches("sha256-").chars().take(32).collect::<String>().replace('/', "_").replace('+', "-")
            }
        } else {
            album_info.torrent_sha256.trim_start_matches("sha256-").chars().take(32).collect::<String>().replace('/', "_").replace('+', "-")
        };
        
        let sanitized_pname = album_info.pname.replace('"', "").trim().replace(' ', "-");
        stage_root.join(format!("{sanitized_pname}-{truncated}"))
    }
}

pub fn resolve_template(template: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let re = regex::Regex::new(r"\$\{([^\}]+)\}").unwrap();
    
    let mut result = template.to_string();
    for caps in re.captures_iter(template) {
        let key = &caps[1];
        if let Some(val) = vars.get(key) {
            result = result.replace(&caps[0], val);
        }
    }
    result
}

pub fn parse_album_nix(path: &Path) -> Result<AlbumInfo> {
    let content = std::fs::read_to_string(path)?;
    let pname = extract_nix_attr(&content, "pname").unwrap_or_else(|| "unknown".to_string());
    let source_disk = extract_nix_attr(&content, "sourceDisk").unwrap_or_default();
    let torrent_file = extract_nix_attr(&content, "sourceTorrent.file").unwrap_or_default();
    let torrent_sha256 = extract_nix_attr(&content, "sourceTorrent.sha256").unwrap_or_default();

    Ok(AlbumInfo { pname, source_disk, torrent_file, torrent_sha256 })
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

    let torrent_file_path = if album_info.torrent_file.starts_with("./") {
        album_dir.join(album_info.torrent_file.trim_start_matches("./"))
    } else {
        album_dir.join(&album_info.torrent_file)
    };
    
    let source_disk = resolve_source_disk(&album_info, album_dir, &config);

    let torrent_name = Torrent::read_from_file(&torrent_file_path)
        .map_or_else(|_| album_info.pname.clone(), |t| t.name);

    let mut vars = std::collections::HashMap::new();
    vars.insert("sourceDisk".to_string(), source_disk.to_string_lossy().to_string());
    vars.insert("sourceTorrent.file".to_string(), torrent_file_path.to_string_lossy().to_string());
    vars.insert("sourceTorrent.sha256".to_string(), album_info.torrent_sha256);
    vars.insert("sourceTorrent.name".to_string(), torrent_name);

    let cmd_tpl = cmds.get("torrent").context("No 'torrent' command configured")?;
    let final_cmd = resolve_template(cmd_tpl, &vars);

    log::info!("Executing: {final_cmd}");
    let status = std::process::Command::new("sh").arg("-c").arg(&final_cmd).status()?;

    if status.success() {
        log::info!("Download initiated to {}", source_disk.display());
    } else {
        log::error!("Command failed with status: {status}");
    }

    Ok(())
}
