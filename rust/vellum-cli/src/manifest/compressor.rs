use crate::harvest::sanitize_key;
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashSet;
use toml::Value as TomlValue;

pub fn compress(
    mut raw_tracks: Vec<serde_json::Map<String, Value>>,
    manifest_layout: Option<&IndexMap<String, TomlValue>>,
) -> (
    serde_json::Map<String, Value>,
    Vec<serde_json::Map<String, Value>>,
) {
    if raw_tracks.is_empty() {
        return (serde_json::Map::new(), Vec::new());
    }

    let mut forced_track_keys = HashSet::new();
    if let Some(layout) = manifest_layout {
        for (key, meta) in layout {
            if let Some(table) = meta.as_table()
                && table.get("level").and_then(|v| v.as_str()) == Some("track") {
                    forced_track_keys.insert(sanitize_key(key));
                }
        }
    }

    let first_track = &raw_tracks[0];
    let mut candidate_keys: HashSet<String> = first_track.keys().cloned().collect();

    for track in raw_tracks.iter().skip(1) {
        candidate_keys.retain(|k| track.contains_key(k));
    }

    let mut album_pool = serde_json::Map::new();
    let mut keys_to_promote = Vec::new();

    for key in candidate_keys {
        let is_identical = raw_tracks
            .iter()
            .all(|t| t.get(&key) == first_track.get(&key));

        if is_identical {
            let s_key = sanitize_key(&key);
            if forced_track_keys.contains(&s_key) {
                continue;
            }
            keys_to_promote.push(key.clone());
            album_pool.insert(key.clone(), first_track.get(&key).unwrap().clone());
        }
    }

    for track in &mut raw_tracks {
        for k in &keys_to_promote {
            track.remove(k);
        }
    }

    (album_pool, raw_tracks)
}
