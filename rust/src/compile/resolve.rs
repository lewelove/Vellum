use crate::harvest::TrackJson;
use std::collections::HashSet;
use std::path::Path;
use serde_json::{json, Value};
use crate::compile::native_extensions;
use image::DynamicImage;

pub struct AlbumContext<'a> {
    pub source: &'a Value,
    pub tracks: &'a [Value],
    pub album_root: &'a Path,
    pub library_root: &'a Path,
    pub meta_hash: &'a str,
    pub meta_mtime: u64,
    pub cover_hash: &'a str,
    pub cover_path: Option<&'a str>,
    pub cover_mtime: u64,
    pub cover_byte_size: u64,
    pub cover_image: Option<&'a DynamicImage>,
}

pub struct TrackContext<'a> {
    pub ordinal_track_number: u32,
    pub ordinal_disc_number: u32,
    pub _harvest: &'a TrackJson,
    pub source: &'a Value,
    pub album_source: &'a Value,
    pub _album_root: &'a Path,
    pub _library_root: &'a Path,
}

pub fn resolve_album_key(key: &str, ctx: &AlbumContext) -> Option<Value> {
    match key {
        "album" => Some(json!(get_raw(ctx.source, "album", "Unknown Album"))),
        "albumartist" => Some(json!(get_raw(ctx.source, "albumartist", "Unknown Artist"))),
        "date" => Some(json!(native_extensions::resolve_date(ctx))),
        "genre" => Some(json!(native_extensions::resolve_genre(ctx))),
        "comment" => Some(json!(native_extensions::resolve_comment(ctx))),
        "original_yyyy_mm" => Some(json!(native_extensions::resolve_original_yyyy_mm(ctx))),
        "release_yyyy_mm" => Some(json!(native_extensions::resolve_release_yyyy_mm(ctx))),
        _ => native_extensions::resolve_album_key(key, ctx),
    }
}

pub fn resolve_track_key(key: &str, ctx: &TrackContext) -> Option<Value> {
    match key {
        "title" => Some(json!(get_raw(ctx.source, "title", "Untitled"))),
        "artist" => Some(json!(get_raw_with_fallback(ctx.source, ctx.album_source, "artist", "albumartist", "Unknown Artist"))),
        "tracknumber" => Some(json!(ctx.ordinal_track_number)),
        "discnumber" => Some(json!(ctx.ordinal_disc_number)),
        _ => native_extensions::resolve_track_key(key, ctx),
    }
}

pub fn resolve_album_info_duration_ms(ctx: &AlbumContext) -> u64 {
    ctx.tracks.iter().filter_map(|t| t.get("info").and_then(|i| i.get("track_duration")).and_then(serde_json::Value::as_u64)).sum()
}

pub fn resolve_album_info_unix_added(ctx: &AlbumContext) -> u64 {
    let keys = [
        "unix_added_primary",
        "unixtimeyoutube",
        "unixtimeapple",
        "unixtimefoobar",
        "unix_added_youtube",
        "unix_added_applemusic",
        "unix_added_foobar",
        "unix_added_local",
    ];
    for key in keys {
        if let Some(val) = ctx.source.get(key).and_then(Value::as_str)
            && let Ok(ts) = val.parse::<u64>() { return ts; }
    }
    0
}

pub fn get_raw(source: &Value, key: &str, default: &str) -> String {
    source.get(key).and_then(Value::as_str).unwrap_or(default).to_string()
}

pub fn get_raw_with_fallback(source: &Value, album: &Value, key: &str, album_key: &str, default: &str) -> String {
    source.get(key).or_else(|| album.get(album_key)).and_then(Value::as_str).unwrap_or(default).to_string()
}

pub fn format_ms(ms: u64) -> String {
    let s = (ms / 1000) % 60;
    let m = (ms / (1000 * 60)) % 60;
    let h = ms / (1000 * 60 * 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

pub fn calculate_total_discs(tracks: &[Value]) -> u32 {
    let mut discs = HashSet::new();
    for t in tracks {
        let val = match t.get("DISCNUMBER") {
            Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
            Some(Value::String(s)) => s.split('/').next().unwrap_or("0").parse::<u64>().unwrap_or(0),
            _ => 0,
        };
        if val > 0 {
            discs.insert(val);
        }
    }
    if discs.is_empty() { 1 } else { u32::try_from(discs.len()).unwrap_or(u32::MAX) }
}

pub fn rel_path(target: &Path, base: &Path) -> String {
    target.strip_prefix(base).map_or_else(|_| target.to_string_lossy().to_string(), |p| p.to_string_lossy().to_string())
}
