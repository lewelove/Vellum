use serde_json::{Value, json};
use libvellum::error::VellumError;
use std::path::Path;

pub fn sort_json_keys(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = std::mem::take(map).into_iter().collect();
            entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
            for (k, mut v) in entries {
                sort_json_keys(&mut v);
                map.insert(k, v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                sort_json_keys(v);
            }
        }
        _ => {}
    }
}

pub fn is_valid_hex_color(s: &str) -> bool {
    if !s.starts_with('#') {
        return false;
    }
    let hex = &s[1..];
    let len = hex.len();
    if len != 3 && len != 4 && len != 6 && len != 8 {
        return false;
    }
    hex.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn validate_and_format_colors(colors: &Value, album_root: &Path) -> Result<Value, VellumError> {
    let mut formatted_colors = serde_json::Map::new();
    
    if let Some(fg) = colors.get("foreground")
        && let Some(s) = fg.as_str()
    {
        if !is_valid_hex_color(s) {
            return Err(VellumError::InvalidColorFormat {
                path: album_root.to_path_buf(),
                key: "foreground".to_string(),
                found: s.to_string(),
            });
        }
        formatted_colors.insert("foreground".to_string(), json!(s));
    }
    
    if let Some(bg) = colors.get("background") {
        if let Some(arr) = bg.as_array() {
            let mut bg_arr = Vec::new();
            for (i, v) in arr.iter().enumerate() {
                if let Some(s) = v.as_str() {
                    if !is_valid_hex_color(s) {
                        return Err(VellumError::InvalidColorFormat {
                            path: album_root.to_path_buf(),
                            key: format!("background[{i}]"),
                            found: s.to_string(),
                        });
                    }
                    bg_arr.push(s.to_string());
                }
            }
            formatted_colors.insert("background".to_string(), json!(bg_arr));
        } else if let Some(s) = bg.as_str() {
            if !is_valid_hex_color(s) {
                return Err(VellumError::InvalidColorFormat {
                    path: album_root.to_path_buf(),
                    key: "background".to_string(),
                    found: s.to_string(),
                });
            }
            formatted_colors.insert("background".to_string(), json!(vec![s]));
        }
    }

    Ok(Value::Object(formatted_colors))
}
