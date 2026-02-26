use serde_json::Value;

pub fn get_raw(source: &Value, key: &str, default: &str) -> String {
    source
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

pub fn get_raw_with_fallback(
    source: &Value,
    album: &Value,
    key: &str,
    album_key: &str,
    default: &str,
) -> String {
    source
        .get(key)
        .or_else(|| album.get(album_key))
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

pub fn format_ms(ms: u64) -> String {
    let s = (ms / 1000) % 60;
    let m = (ms / (1000 * 60)) % 60;
    let h = ms / (1000 * 60 * 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
