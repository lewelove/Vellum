use serde_json::Value;

pub fn calculate_file_tag_subset_match(enriched: &Value, harvest: &[Value]) -> bool {
    let Some(album_obj) = enriched.get("album").and_then(|v| v.as_object()) else { return false; };
    let Some(tracks_arr) = enriched.get("tracks").and_then(|v| v.as_array()) else { return false; };

    if tracks_arr.len() != harvest.len() { return false; }

    let total_discs: u64 = album_obj.get("total_discs")
        .and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(1);

    for (idx, compiled_track) in tracks_arr.iter().enumerate() {
        let Some(t_obj) = compiled_track.as_object() else { return false; };
        let Some(phys_tags) = harvest[idx].get("tags").and_then(|v| v.as_object()) else { return false; };

        // Check Album-level tags that should be propagated
        for (k, v) in album_obj {
            if is_standard_tag_key(k) && !v.is_null() {
                let p_val = phys_tags.get(k).and_then(|v| v.as_str()).unwrap_or("");
                if !compare_values(k, v, p_val, total_discs) { return false; }
            }
        }
        
        // Check Track-level tags
        for (k, v) in t_obj {
            if is_standard_tag_key(k) && !v.is_null() {
                let p_val = phys_tags.get(k).and_then(|v| v.as_str()).unwrap_or("");
                if !compare_values(k, v, p_val, total_discs) { return false; }
            }
        }
    }
    true
}

fn is_standard_tag_key(key: &str) -> bool {
    // Only accept keys that consist of Uppercase letters, Numbers, or Underscores.
    // This excludes internal helpers (lowercase), special fields, or empty keys.
    if key.is_empty() { return false; }
    
    // Explicit exclusions for calculated/internal fields that happen to be uppercase
    if key == "REPLAYGAIN_TRACK_GAIN" || key == "REPLAYGAIN_TRACK_PEAK" || 
       key == "REPLAYGAIN_ALBUM_GAIN" || key == "REPLAYGAIN_ALBUM_PEAK" {
           return false;
    }

    key.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

fn compare_values(key: &str, compiled: &Value, physical: &str, total_discs: u64) -> bool {
    if key == "DISCNUMBER" && total_discs == 1 { return true; }

    let s_comp = match compiled {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr.iter()
            .map(|v| v.as_str().unwrap_or("").trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>().join("; "),
        _ => compiled.to_string().replace('"', ""),
    };

    let (s_c, s_p) = (s_comp.trim(), physical.trim());
    
    if key == "TRACKNUMBER" || key == "DISCNUMBER" {
        let parse = |s: &str| s.split('/').next().unwrap_or("0").parse::<u64>().unwrap_or(0);
        return parse(s_c) == parse(s_p);
    }
    
    s_c == s_p
}
