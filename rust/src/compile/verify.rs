use serde_json::Value;
use std::collections::HashMap;

pub fn calculate_file_tag_subset_match(enriched: &Value, harvest: &[Value], registry: &HashMap<String, Value>) -> bool {
    let Some(album_obj) = enriched.get("album").and_then(|v| v.as_object()) else { return false; };
    let Some(tracks_arr) = enriched.get("tracks").and_then(|v| v.as_array()) else { return false; };
    if tracks_arr.len() != harvest.len() { return false; }

    let album_sync_keys: Vec<String> = registry.iter()
        .filter(|(_, m)| m.get("level").and_then(|l| l.as_str()) == Some("album") && m.get("sync").and_then(|s| s.as_bool()).unwrap_or(true))
        .map(|(k, _)| k.clone())
        .collect();

    let track_sync_keys: Vec<String> = registry.iter()
        .filter(|(_, m)| m.get("level").and_then(|l| l.as_str()) == Some("tracks") && m.get("sync").and_then(|s| s.as_bool()).unwrap_or(true))
        .map(|(k, _)| k.clone())
        .collect();

    for (idx, compiled_track) in tracks_arr.iter().enumerate() {
        let Some(t_obj) = compiled_track.as_object() else { return false; };
        let Some(p_tags) = harvest[idx].get("tags").and_then(|v| v.as_object()) else { return false; };

        for k in &album_sync_keys {
            if let Some(v) = album_obj.get(k) {
                let p_val = p_tags.get(&k.to_uppercase()).and_then(|v| v.as_str()).unwrap_or("");
                if !compare_values(k, v, p_val) { return false; }
            }
        }

        for k in &track_sync_keys {
            if let Some(v) = t_obj.get(k) {
                let p_val = p_tags.get(&k.to_uppercase()).and_then(|v| v.as_str()).unwrap_or("");
                if !compare_values(k, v, p_val) { return false; }
            }
        }
    }
    true
}

fn compare_values(key: &str, compiled: &Value, physical: &str) -> bool {
    let s_comp = match compiled {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr.iter()
            .map(|v| v.as_str().unwrap_or("").trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>().join("; "),
        _ => compiled.to_string().replace('"', ""),
    };
    let (s_c, s_p) = (s_comp.trim(), physical.trim());
    if key == "tracknumber" || key == "discnumber" {
        let parse = |s: &str| s.split('/').next().unwrap_or("0").parse::<u64>().unwrap_or(0);
        return parse(s_c) == parse(s_p);
    }
    s_c == s_p
}
