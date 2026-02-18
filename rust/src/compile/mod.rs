use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use serde_json::json;

use crate::harvest;

pub fn run(target_path: PathBuf) -> Result<()> {
    let project_root = find_project_root().context("Could not find project root (config.toml)")?;
    let config_path = project_root.join("config.toml");
    let config_content = fs::read_to_string(&config_path).context("Failed to read config.toml")?;
    let config_toml: toml::Value = toml::from_str(&config_content)?;

    let metadata_path = if target_path.is_dir() {
        target_path.join("metadata.toml")
    } else {
        target_path.clone()
    };

    if !metadata_path.exists() {
        anyhow::bail!("Metadata file not found: {:?}", metadata_path);
    }

    let album_root = metadata_path.parent().unwrap().to_path_buf();
    let metadata_content = fs::read_to_string(&metadata_path)?;
    let metadata_toml: toml::Value = toml::from_str(&metadata_content)?;

    let gen_cfg = config_toml.get("generate").context("Missing [generate] in config")?;
    let supported_exts: Vec<String> = gen_cfg.get("supported_extensions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec![".flac".to_string()]);

    let extensions: Vec<&str> = supported_exts.iter().map(|s| s.as_str()).collect();
    
    let mut harvested_files = Vec::new();
    let audio_files = scan_audio_files(&album_root, &extensions);
    for path in audio_files {
        if let Ok(data) = harvest::harvest_file(&path) {
            harvested_files.push(data);
        }
    }

    let manifest = json!({
        "config": config_toml,
        "metadata": metadata_toml,
        "harvest": harvested_files,
        "paths": {
            "album_root": album_root,
            "project_root": project_root,
        }
    });

    let mut child = Command::new("bun")
        .arg("run")
        .arg(project_root.join("stdlib/compiler_kernel.js"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn Bun. Ensure bun is installed.")?;

    let mut stdin = child.stdin.take().context("Failed to open stdin")?;
    stdin.write_all(serde_json::to_string(&manifest)?.as_bytes())?;
    drop(stdin);

    let output = child.wait_with_output()?;
    if !output.status.success() {
        anyhow::bail!("Bun compiler kernel exited with error");
    }

    io::stdout().write_all(&output.stdout)?;

    Ok(())
}

fn scan_audio_files(root: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| extensions.contains(&format!(".{}", ext.to_lowercase()).as_str()))
                .unwrap_or(false)
        })
        .collect();

    files.sort_by(|a, b| {
        alphanumeric_sort::compare_path(a, b)
    });
    
    files
}

fn find_project_root() -> Option<PathBuf> {
    let mut curr = std::env::current_dir().ok()?;
    loop {
        if curr.join("config.toml").exists() {
            return Some(curr);
        }
        if let Some(parent) = curr.parent() {
            curr = parent.to_path_buf();
        } else {
            return None;
        }
    }
}
