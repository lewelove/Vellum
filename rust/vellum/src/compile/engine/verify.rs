use serde_json::Value;

pub fn calculate_file_tag_subset_match(
    enriched: &Value,
    harvest: &[Value],
    subset_keys: &[String],
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

    for (idx, compiled_track) in tracks_arr.iter().enumerate() {
        let Some(t_obj) = compiled_track.as_object() else {
            return false;
        };
        let Some(p_tags) = harvest[idx].get("tags").and_then(Value::as_object) else {
            return false;
        };

        for key in subset_keys {
            let k_lower = key.to_lowercase();

            let mut compiled_val = t_obj.get(&k_lower);
            if compiled_val.is_none() {
                compiled_val = t_obj.get("tags").and_then(|tags| tags.get(&k_lower));
            }
            if compiled_val.is_none() {
                compiled_val = album_obj.get(&k_lower);
            }
            if compiled_val.is_none() {
                compiled_val = album_obj.get("tags").and_then(|tags| tags.get(&k_lower));
            }

            if let Some(v) = compiled_val {
                let p_val = p_tags
                    .get(&k_lower)
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if !compare_values(&k_lower, v, p_val) {
                    return false;
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
        let parse = |s: &str| {
            s.split('/')
                .next()
                .unwrap_or("0")
                .trim()
                .parse::<u64>()
                .unwrap_or(0)
        };
        return parse(s_c) == parse(s_p);
    }
    s_c == s_p
}

