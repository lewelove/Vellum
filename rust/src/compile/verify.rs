use serde_json::Value;
use std::collections::HashMap;

pub fn calculate_file_tag_subset_match(
    enriched: &Value,
    harvest: &[Value],
    registry: &HashMap<String, Value>,
) -> bool {
    let Some(album_obj) = enriched.get("album").and_then(Value::as_object) else {
        return false;
    };
    let Some(tracks_arr) = enriched.get("tracks").and_then(Value::as_array) else {
        return false;
    };
    if tracks_arr.len() != harvest.len() {
        return false;
    }

    let album_core_keys = ["ALBUM", "ALBUMARTIST", "DATE", "GENRE", "COMMENT"];

    let track_core_keys = ["TITLE", "ARTIST", "TRACKNUMBER", "DISCNUMBER"];

    for (idx, compiled_track) in tracks_arr.iter().enumerate() {
        let Some(t_obj) = compiled_track.as_object() else {
            return false;
        };
        let Some(p_tags) = harvest[idx].get("tags").and_then(Value::as_object) else {
            return false;
        };

        for k in &album_core_keys {
            if let Some(v) = album_obj.get(*k) {
                let p_val = p_tags.get(&k.to_uppercase()).and_then(Value::as_str).unwrap_or("");
                if !compare_values(k, v, p_val) {
                    return false;
                }
            }
        }

        for k in &track_core_keys {
            if let Some(v) = t_obj.get(*k) {
                let p_val = p_tags.get(&k.to_uppercase()).and_then(Value::as_str).unwrap_or("");
                if !compare_values(k, v, p_val) {
                    return false;
                }
            }
        }

        if let Some(a_tags) = album_obj.get("tags").and_then(Value::as_object) {
            for (key, meta) in registry {
                if meta.get("level").and_then(Value::as_str) != Some("album")
                    || meta.get("sync").and_then(Value::as_bool) == Some(false)
                {
                    continue;
                }
                if let Some(v) = a_tags.get(&key.to_uppercase()) {
                    let p_val =
                        p_tags.get(&key.to_uppercase()).and_then(Value::as_str).unwrap_or("");
                    if !compare_values(key, v, p_val) {
                        return false;
                    }
                }
            }
        }

        if let Some(t_tags) = t_obj.get("tags").and_then(Value::as_object) {
            for (key, meta) in registry {
                if meta.get("level").and_then(Value::as_str) != Some("tracks")
                    || meta.get("sync").and_then(Value::as_bool) == Some(false)
                {
                    continue;
                }
                if let Some(v) = t_tags.get(&key.to_uppercase()) {
                    let p_val =
                        p_tags.get(&key.to_uppercase()).and_then(Value::as_str).unwrap_or("");
                    if !compare_values(key, v, p_val) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn compare_values(key: &str, compiled: &Value, physical: &str) -> bool {
    let s_comp = match compiled {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr
            .iter()
            .map(|v| v.as_str().unwrap_or("").trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("; "),
        Value::Null => return physical.is_empty(),
        _ => compiled.to_string().replace('"', ""),
    };
    let (s_c, s_p) = (s_comp.trim(), physical.trim());
    let k_lower = key.to_lowercase();
    if k_lower == "tracknumber" || k_lower == "discnumber" {
        let parse = |s: &str| s.split('/').next().unwrap_or("0").parse::<u64>().unwrap_or(0);
        return parse(s_c) == parse(s_p);
    }
    s_c == s_p
}
