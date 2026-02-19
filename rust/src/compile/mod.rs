mod nix;
pub mod resolve;

use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;
use serde_json::{json, Value};
use sha2::{Sha256, Digest};
use xxhash_rust::xxh64::xxh64;

use crate::harvest;
use crate::compile::nix::get_nix_env;
use crate::compile::resolve::{resolve_standard_track, resolve_standard_album};

pub fn run(target_path: PathBuf, json_output: bool) -> Result<()> {
    let project_root = find_project_root().context("Could not find project root (config.toml)")?;
    let config_path = project_root.join("config.toml");
    let config_content = fs::read_to_string(&config_path).context("Failed to read config.toml")?;
    let mut config_toml: toml::Value = toml::from_str(&config_content)?;

    let home = dirs::home_dir().context("Could not determine home directory")?;
    let ext_folder_path = config_toml.get("compiler")
        .and_then(|c| c.get("extensions_folder"))
        .and_then(|v| v.as_str())
        .map(|s| expand_path_with_home(s, &home));

    let metadata_path = if target_path.is_dir() {
        target_path.join("metadata.toml")
    } else {
        target_path.clone()
    };

    if !metadata_path.exists() {
        anyhow::bail!("Metadata file not found: {:?}", metadata_path);
    }

    let album_root = metadata_path.parent().unwrap().to_path_buf();
    let metadata_mtime = fs::metadata(&metadata_path)?.modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    
    let mut metadata_file = fs::File::open(&metadata_path)?;
    let mut metadata_buffer = Vec::new();
    metadata_file.read_to_end(&mut metadata_buffer)?;
    
    let metadata_hash = {
        let mut hasher = Sha256::new();
        hasher.update(&metadata_buffer);
        format!("{:x}", hasher.finalize())
    };

    let metadata_toml_raw: toml::Value = toml::from_str(&String::from_utf8_lossy(&metadata_buffer))?;
    let metadata_json = toml_to_json(metadata_toml_raw);

    let gen_cfg = config_toml.get("generate").context("Missing [generate] in config")?;
    let supported_exts: Vec<String> = gen_cfg.get("supported_extensions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec![".flac".to_string()]);

    let extensions: Vec<&str> = supported_exts.iter().map(|s| s.as_str()).collect();
    let audio_files = scan_audio_files(&album_root, &extensions);
    
    let mut harvested_files = Vec::new();
    let mut standard_tracks = Vec::new();
    
    let album_source = metadata_json.get("album").cloned().unwrap_or(json!({}));
    let track_entries = metadata_json.get("tracks").and_then(|v| v.as_array());

    for (idx, path) in audio_files.iter().enumerate() {
        if let Ok(data) = harvest::harvest_file(path) {
            let mut h_item = serde_json::to_value(&data)?;
            if let Ok(rel) = path.strip_prefix(&album_root) {
                h_item["track_path"] = json!(rel.to_string_lossy());
            }

            let entry = track_entries.and_then(|arr| arr.get(idx)).cloned().unwrap_or(json!({}));
            let resolved = resolve_standard_track(idx, &data, &entry, &album_source);
            
            harvested_files.push(h_item);
            standard_tracks.push(resolved);
        }
    }

    let standard_album = resolve_standard_album(&album_source, &standard_tracks);

    // Cover Processing
    let mut cover_hash = String::new();
    let cover_path_candidates = ["cover.jpg", "cover.png", "folder.jpg", "front.jpg"];
    for cand in cover_path_candidates {
        let p = album_root.join(cand);
        if p.exists() {
            if let Ok(buf) = fs::read(&p) {
                let hash = xxh64(&buf, 0);
                cover_hash = base64::display::Base64Display::new(
                    &hash.to_be_bytes(), 
                    &base64::engine::general_purpose::URL_SAFE_NO_PAD
                ).to_string();
                break;
            }
        }
    }

    let payload = json!({
        "config": toml_to_json(config_toml),
        "metadata": metadata_json,
        "harvest": harvested_files,
        "standard": {
            "album": standard_album,
            "tracks": standard_tracks
        },
        "paths": {
            "album_root": album_root,
            "project_root": project_root,
            "metadata_toml_mtime": metadata_mtime,
            "metadata_toml_hash": metadata_hash,
            "cover_hash": cover_hash
        }
    });

    let mut nix_env = get_nix_env(&project_root, ext_folder_path.as_deref())?;
    nix_env.insert("HOME".to_string(), home.to_string_lossy().to_string());
    
    let kernel_path = project_root.join("stdlib/compiler_kernel.js");

    let mut child = Command::new("bun")
        .arg("run")
        .arg(&kernel_path)
        .envs(nix_env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn Bun compiler kernel.")?;

    let mut stdin = child.stdin.take().context("Failed to open stdin")?;
    stdin.write_all(serde_json::to_string(&payload)?.as_bytes())?;
    drop(stdin);

    let output = child.wait_with_output()?;
    if !output.status.success() {
        anyhow::bail!("Bun compiler kernel exited with error");
    }

    let mut enriched: Value = serde_json::from_slice(&output.stdout)?;
    
    let is_match = calculate_file_tag_subset_match(&enriched, &harvested_files);
    if let Some(album) = enriched.get_mut("album").and_then(|v| v.as_object_mut()) {
        album.insert("file_tag_subset_match".to_string(), json!(is_match));
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&enriched)?);
    } else {
        let lock_json_path = album_root.join("metadata.lock.json");
        fs::write(lock_json_path, serde_json::to_string_pretty(&enriched)?)?;
    }

    Ok(())
}

fn calculate_file_tag_subset_match(enriched: &Value, harvest: &[Value]) -> bool {
    let Some(album_obj) = enriched.get("album").and_then(|v| v.as_object()) else { return false; };
    let Some(tracks_arr) = enriched.get("tracks").and_then(|v| v.as_array()) else { return false; };

    if tracks_arr.len() != harvest.len() { return false; }

    let total_discs: u64 = album_obj.get("total_discs")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    for (idx, compiled_track) in tracks_arr.iter().enumerate() {
        let Some(t_obj) = compiled_track.as_object() else { return false; };
        let h_obj = &harvest[idx];
        let Some(physical_tags) = h_obj.get("tags").and_then(|v| v.as_object()) else { return false; };

        for (key, val) in album_obj {
            if !is_tag_key(key) || val.is_null() { continue; }
            let p_val = physical_tags.get(key).and_then(|v| v.as_str()).unwrap_or("");
            if !compare_values(key, val, p_val, total_discs) { return false; }
        }

        for (key, val) in t_obj {
            if !is_tag_key(key) || val.is_null() { continue; }
            let p_val = physical_tags.get(key).and_then(|v| v.as_str()).unwrap_or("");
            if !compare_values(key, val, p_val, total_discs) { return false; }
        }
    }

    true
}

fn is_tag_key(key: &str) -> bool {
    key.chars().all(|c| !c.is_alphabetic() || c.is_uppercase())
}

fn compare_values(key: &str, compiled: &Value, physical: &str, total_discs: u64) -> bool {
    if key == "DISCNUMBER" && total_discs == 1 { return true; }

    let s_compiled = match compiled {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr.iter()
            .map(|v| v.as_str().unwrap_or("").trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("; "),
        _ => compiled.to_string().replace('"', ""),
    };

    let s_comp = s_compiled.trim();
    let s_phys = physical.trim();

    if key == "TRACKNUMBER" || key == "DISCNUMBER" {
        let parse = |s: &str| s.split('/').next().unwrap_or("0").parse::<u64>().unwrap_or(0);
        return parse(s_comp) == parse(s_phys);
    }

    s_comp == s_phys
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

    files.sort_by(|a, b| alphanumeric_sort::compare_path(a, b));
    files
}

fn find_project_root() -> Option<PathBuf> {
    let mut curr = std::env::current_dir().ok()?;
    loop {
        if curr.join("config.toml").exists() { return Some(curr); }
        if let Some(parent) = curr.parent() { curr = parent.to_path_buf(); } else { return None; }
    }
}

fn expand_path_with_home(path_str: &str, home: &Path) -> PathBuf {
    if path_str.starts_with('~') {
        if path_str == "~" { return home.to_path_buf(); }
        if let Some(stripped) = path_str.strip_prefix("~/") { return home.join(stripped); }
    }
    PathBuf::from(path_str)
}

fn toml_to_json(toml: toml::Value) -> Value {
    match toml {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Integer(i) => Value::Number(i.into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Datetime(d) => Value::String(d.to_string()),
        toml::Value::Array(a) => Value::Array(a.into_iter().map(toml_to_json).collect()),
        toml::Value::Table(t) => {
            let mut map = serde_json::Map::new();
            for (k, v) in t { map.insert(k, toml_to_json(v)); }
            Value::Object(map)
        }
    }
}
