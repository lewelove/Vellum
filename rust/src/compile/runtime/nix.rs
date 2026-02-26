use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

pub fn get_nix_env(
    project_root: &Path,
    explicit_flake: Option<PathBuf>,
) -> Result<HashMap<String, String>> {
    let target = explicit_flake.map_or_else(
        || project_root.to_path_buf(),
        |p| p.parent().unwrap_or(project_root).to_path_buf(),
    );
    let flake_file = target.join("flake.nix");
    if !flake_file.exists() {
        return Ok(HashMap::new());
    }

    let cache_dir = dirs::home_dir().context("No home")?.join(".vellum/cache");
    fs::create_dir_all(&cache_dir)?;
    let mtime = fs::metadata(&flake_file)?.modified()?;
    let cache_key = format!(
        "{:?}",
        mtime.duration_since(SystemTime::UNIX_EPOCH)?.as_secs()
    );
    let cache_file = cache_dir.join("nix_env.json");

    if let Ok(content) = fs::read_to_string(&cache_file)
        && let Ok(cache) = serde_json::from_str::<serde_json::Value>(&content)
        && cache.get("key").and_then(Value::as_str) == Some(&cache_key)
    {
        let mut map = HashMap::new();
        if let Some(vars) = cache.get("variables").and_then(Value::as_object) {
            for (k, v) in vars {
                if let Some(s) = v.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
        }
        return Ok(map);
    }

    let output = Command::new("nix")
        .args([
            "print-dev-env",
            "--extra-experimental-features",
            "nix-command",
            "--extra-experimental-features",
            "flakes",
            "--impure",
            "--json",
            ".",
        ])
        .current_dir(&target)
        .output()?;

    let mut map = HashMap::new();
    let mut cache_map = serde_json::Map::new();
    if output.status.success()
        && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        && let Some(vars) = val.get("variables").and_then(Value::as_object)
    {
        for (k, v) in vars {
            if let Some(s) = v.get("value").and_then(Value::as_str) {
                map.insert(k.clone(), s.to_string());
                cache_map.insert(k.clone(), serde_json::Value::String(s.to_string()));
            }
        }
    }
    let _ = fs::write(
        cache_file,
        serde_json::to_string(&serde_json::json!({"key": cache_key, "variables": cache_map}))?,
    );
    Ok(map)
}
