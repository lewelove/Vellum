use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

pub fn get_nix_env(project_root: &Path, extensions_folder: Option<&Path>) -> Result<HashMap<String, String>> {
    // 1. Priority: The Extensions Folder Flake (The Runtime Environment)
    let ext_flake = extensions_folder.map(|p| p.join("flake.nix"));
    
    // 2. Fallback: The Project Root Flake (The Dev Environment)
    let root_flake = project_root.join("flake.nix");
    
    // 3. Fallback: The User Global Flake
    let global_flake = dirs::home_dir().map(|h| h.join(".vellum/extensions/flake.nix"));

    let target_flake_path = if let Some(p) = ext_flake.filter(|p| p.exists()) {
        p
    } else if root_flake.exists() {
        root_flake
    } else if let Some(p) = global_flake.filter(|p| p.exists()) {
        p
    } else {
        // No flake found? Return empty env (rely on system PATH)
        return Ok(HashMap::new());
    };

    let flake_dir = target_flake_path.parent().context("Invalid flake path")?;
    let cache_dir = dirs::home_dir().context("No home dir")?.join(".vellum/cache");
    fs::create_dir_all(&cache_dir)?;

    let lock_path = flake_dir.join("flake.lock");
    let mtime = fs::metadata(&target_flake_path)?.modified()?;
    let lock_mtime = fs::metadata(&lock_path).map(|m| m.modified().unwrap_or(mtime)).unwrap_or(mtime);
    
    let cache_key = format!(
        "{:?}-{:?}", 
        mtime.duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
        lock_mtime.duration_since(SystemTime::UNIX_EPOCH)?.as_secs()
    );
    
    let cache_file = cache_dir.join("nix_env.json");

    if cache_file.exists() {
        if let Ok(content) = fs::read_to_string(&cache_file) {
            let cache: serde_json::Value = serde_json::from_str(&content)?;
            if cache.get("key").and_then(|k| k.as_str()) == Some(&cache_key) {
                if let Some(vars) = cache.get("variables").and_then(|v| v.as_object()) {
                    let mut env_map = HashMap::new();
                    for (k, v) in vars {
                        if let Some(s) = v.as_str() {
                            env_map.insert(k.clone(), s.to_string());
                        }
                    }
                    return Ok(env_map);
                }
            }
        }
    }

    let output = Command::new("nix")
        .arg("print-dev-env")
        .arg("--json")
        .arg(flake_dir)
        .output()
        .context("Failed to execute nix print-dev-env")?;

    if !output.status.success() {
        return Ok(HashMap::new());
    }

    let val: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let mut env_map = HashMap::new();
    let mut cache_map = serde_json::Map::new();

    if let Some(variables) = val.get("variables").and_then(|v| v.as_object()) {
        for (k, v) in variables {
            if let Some(val_str) = v.get("value").and_then(|s| s.as_str()) {
                env_map.insert(k.clone(), val_str.to_string());
                cache_map.insert(k.clone(), serde_json::Value::String(val_str.to_string()));
            }
        }
    }

    let full_cache = serde_json::json!({
        "key": cache_key,
        "variables": cache_map
    });

    fs::write(cache_file, serde_json::to_string(&full_cache)?)?;

    Ok(env_map)
}
