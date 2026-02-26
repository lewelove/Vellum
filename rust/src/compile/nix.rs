use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

pub fn get_nix_env(project_root: &Path, explicit_flake: Option<PathBuf>) -> Result<HashMap<String, String>> {
    let target_flake_dir = explicit_flake.map_or_else(|| project_root.to_path_buf(), |flake_path| if flake_path.exists() {
        flake_path.parent().unwrap_or(project_root).to_path_buf()
    } else {
        project_root.to_path_buf()
    });

    let flake_file = target_flake_dir.join("flake.nix");
    if !flake_file.exists() {
        return Ok(HashMap::new());
    }

    let cache_dir = dirs::home_dir().context("No home dir")?.join(".vellum/cache");
    fs::create_dir_all(&cache_dir)?;

    let mtime = fs::metadata(&flake_file)?.modified()?;
    let cache_key = format!("{:?}", mtime.duration_since(SystemTime::UNIX_EPOCH)?.as_secs());
    let cache_file = cache_dir.join("nix_env.json");

    if let Ok(content) = fs::read_to_string(&cache_file)
        && let Ok(cache) = serde_json::from_str::<serde_json::Value>(&content)
        && cache.get("key").and_then(serde_json::Value::as_str) == Some(&cache_key) {
        let mut env_map = HashMap::new();
        if let Some(vars) = cache.get("variables").and_then(serde_json::Value::as_object) {
            for (k, v) in vars {
                if let Some(s) = v.as_str() { env_map.insert(k.clone(), s.to_string()); }
            }
        }
        return Ok(env_map);
    }

    log::info!("Resolving Nix environment from {target_flake_dir:?}");

    let output = Command::new("nix")
        .args([
            "print-dev-env",
            "--extra-experimental-features", "nix-command",
            "--extra-experimental-features", "flakes",
            "--impure",
            "--json",
            ".",
        ])
        .current_dir(&target_flake_dir)
        .output()
        .context("Failed to execute nix print-dev-env")?;

    let mut env_map = HashMap::new();
    let mut cache_map = serde_json::Map::new();

    if output.status.success()
        && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        && let Some(variables) = val.get("variables").and_then(serde_json::Value::as_object) {
        for (k, v) in variables {
            if let Some(val_str) = v.get("value").and_then(serde_json::Value::as_str) {
                env_map.insert(k.clone(), val_str.to_string());
                cache_map.insert(k.clone(), serde_json::Value::String(val_str.to_string()));
            }
        }
    } else {
        log::warn!("Nix resolution failed. Falling back to system environment. (Result cached)");
    }

    // Cache the result (even if empty) to prevent repeated 4s resolution attempts
    let _ = fs::write(cache_file, serde_json::to_string(&serde_json::json!({ 
        "key": cache_key, 
        "variables": cache_map 
    }))?);

    Ok(env_map)
}
