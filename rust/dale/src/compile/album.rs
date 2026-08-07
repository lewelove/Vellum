use libdale::error::DaleError;
use serde_json::{Value, json};
use std::path::Path;
use std::collections::BTreeMap;

pub fn is_virtual_album(album_root: &Path) -> bool {
    let virtual_path = album_root.join("virtual.toml");
    if let Ok(content) = std::fs::read_to_string(&virtual_path)
        && let Ok(parsed) = toml::from_str::<toml::Value>(&content)
        && let Some(album) = parsed.get("album")
        && let Some(virt) = album.get("virtual").and_then(toml::Value::as_bool)
    {
        return virt;
    }
    false
}

pub fn parse_mandatory_album_fields(
    primary_album: &Value,
    album_root: &Path,
) -> Result<(String, String, String), DaleError> {
    let get_album_str = |k: &str| -> Result<String, DaleError> {
        let v = primary_album.get(k);
        if let Some(s) = v.and_then(Value::as_str) {
            if !s.is_empty() { return Ok(s.to_string()); }
        } else if let Some(n) = v.and_then(Value::as_number) {
            return Ok(n.to_string());
        }
        Err(DaleError::TypeMismatch {
            path: album_root.to_path_buf(),
            key: k.to_string(),
            expected_type: "string".to_string(),
            found_val: "missing or empty".to_string(),
        })
    };

    let albumartist = get_album_str("albumartist").or_else(|_| get_album_str("artist"))?;
    let album = get_album_str("album")?;
    let date = get_album_str("date")?;
    Ok((albumartist, album, date))
}

pub fn generate_lock_manifests(
    parsed_manifests: &serde_json::Map<String, Value>,
    album_root: &Path,
) -> BTreeMap<String, Value> {
    let mut lock_manifests = BTreeMap::new();
    for (name, _) in parsed_manifests {
        let file_name = format!("{name}.toml");
        let abs_p = album_root.join(&file_name);
        if let Ok(info) = libdale::utils::get_file_info(&abs_p, &file_name, false) {
            lock_manifests.insert(name.clone(), json!({ "file": info }));
        }
    }
    lock_manifests
}
