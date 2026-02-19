use crate::harvest::TrackJson;
use std::collections::HashMap;
use serde_json::{json, Value};

pub fn resolve_standard_track(
    idx: usize,
    harvest: &TrackJson,
    source: &Value,
    album_source: &Value,
) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    let get_source = |key: &str| {
        source.get(key)
            .or_else(|| album_source.get(key))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };

    map.insert("TITLE".to_string(), json!(get_source("TITLE").unwrap_or_else(|| "Untitled".to_string())));
    map.insert("ARTIST".to_string(), json!(get_source("ARTIST").unwrap_or_else(|| get_source("ALBUMARTIST").unwrap_or_else(|| "Unknown Artist".to_string()))));
    map.insert("TRACKNUMBER".to_string(), json!(get_source("TRACKNUMBER").unwrap_or_else(|| (idx + 1).to_string())));
    map.insert("DISCNUMBER".to_string(), json!(get_source("DISCNUMBER").unwrap_or_else(|| "1".to_string())));

    map.insert("track_duration_in_ms".to_string(), json!(harvest.physics.duration_ms));
    map.insert("track_duration_time".to_string(), json!(format_ms(harvest.physics.duration_ms)));
    map.insert("sample_rate".to_string(), json!(harvest.physics.sample_rate));
    map.insert("bits_per_sample".to_string(), json!(harvest.physics.bit_depth.unwrap_or(0)));
    map.insert("channels".to_string(), json!(harvest.physics.channels));
    map.insert("track_size".to_string(), json!(harvest.physics.file_size));
    map.insert("track_mtime".to_string(), json!(harvest.physics.mtime));
    
    map
}

pub fn resolve_standard_album(
    album_source: &Value,
    tracks: &[HashMap<String, Value>],
) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    let get_source = |key: &str| {
        album_source.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    map.insert("ALBUM".to_string(), json!(get_source("ALBUM").unwrap_or_else(|| "Unknown Album".to_string())));
    map.insert("ALBUMARTIST".to_string(), json!(get_source("ALBUMARTIST").unwrap_or_else(|| "Unknown Artist".to_string())));
    map.insert("DATE".to_string(), json!(get_source("DATE").or_else(|| get_source("YEAR")).unwrap_or_else(|| "0000".to_string())));
    
    let total_ms: u64 = tracks.iter()
        .filter_map(|t| t.get("track_duration_in_ms").and_then(|v| v.as_u64()))
        .sum();

    map.insert("album_duration_in_ms".to_string(), json!(total_ms));
    map.insert("album_duration_time".to_string(), json!(format_ms(total_ms)));
    map.insert("total_tracks".to_string(), json!(tracks.len().to_string()));

    map
}

fn format_ms(ms: u64) -> String {
    let seconds = ms / 1000;
    let m = seconds / 60;
    let s = seconds % 60;
    let h = m / 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m % 60, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}
