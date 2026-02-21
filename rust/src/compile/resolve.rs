use crate::harvest::TrackJson;
use std::collections::HashSet;
use std::path::Path;
use serde_json::{json, Value};
use chrono::{TimeZone, Local};
use image::DynamicImage;
use crate::compile::native_extensions;

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
    pub harvest: &'a TrackJson,
    pub source: &'a Value,
    pub album_source: &'a Value,
    pub album_root: &'a Path,
    pub library_root: &'a Path,
}

pub fn resolve_album_standard(key: &str, ctx: &AlbumContext) -> Option<Value> {
    let core = match key {
        "ALBUM" => Some(json!(get_raw(ctx.source, "ALBUM", "Unknown Album"))),
        "ALBUMARTIST" => Some(json!(get_raw(ctx.source, "ALBUMARTIST", "Unknown Artist"))),
        "ORIGINAL_DATE" => Some(json!(format_human_date(&native_extensions::resolve_album_tag("ORIGINAL_YYYY_MM", ctx)?.as_str()?))),
        "RELEASE_DATE" => Some(json!(format_human_date(&native_extensions::resolve_album_tag("RELEASE_YYYY_MM", ctx)?.as_str()?))),
        "total_tracks" => Some(json!(ctx.tracks.len().to_string())),
        "total_discs" => Some(json!(calculate_total_discs(ctx.tracks).to_string())),
        "album_root_path" => Some(json!(rel_path(ctx.album_root, ctx.library_root))),
        "cover_hash" => Some(json!(ctx.cover_hash)),
        "cover_path" => Some(json!(ctx.cover_path.unwrap_or("default_cover.png"))),
        "cover_mtime" => Some(json!(ctx.cover_mtime)),
        "cover_byte_size" => Some(json!(ctx.cover_byte_size)),
        "metadata_toml_hash" => Some(json!(ctx.meta_hash)),
        "metadata_toml_mtime" => Some(json!(ctx.meta_mtime)),
        _ => None,
    };

    if core.is_some() {
        return core;
    }

    native_extensions::resolve_album_tag(key, ctx)
        .or_else(|| native_extensions::resolve_album_helper(key, ctx))
}

pub fn resolve_track_standard(key: &str, ctx: &TrackContext) -> Option<Value> {
    let core = match key {
        "TITLE" => Some(json!(get_raw(ctx.source, "TITLE", "Untitled"))),
        "ARTIST" => Some(json!(get_raw_with_fallback(ctx.source, ctx.album_source, "ARTIST", "ALBUMARTIST", "Unknown Artist"))),
        "TRACKNUMBER" => Some(json!(ctx.ordinal_track_number.to_string())),
        "DISCNUMBER" => Some(json!(ctx.ordinal_disc_number.to_string())),
        "LYRICS" => Some(json!(get_raw(ctx.source, "LYRICS", ""))),
        "track_duration_in_ms" => Some(json!(ctx.harvest.physics.duration_ms)),
        "track_duration_time" => Some(json!(format_ms(ctx.harvest.physics.duration_ms))),
        "sample_rate" => Some(json!(ctx.harvest.physics.sample_rate)),
        "bits_per_sample" => Some(json!(ctx.harvest.physics.bit_depth.unwrap_or(0))),
        "channels" => Some(json!(ctx.harvest.physics.channels)),
        "track_size" => Some(json!(ctx.harvest.physics.file_size)),
        "track_mtime" => Some(json!(ctx.harvest.physics.mtime)),
        "track_library_path" => Some(json!(rel_path(&ctx.harvest.path, ctx.library_root))),
        "track_path" => Some(json!(rel_path(&ctx.harvest.path, ctx.album_root))),
        _ => None,
    };

    if core.is_some() {
        return core;
    }

    native_extensions::resolve_track_tag(key, ctx)
        .or_else(|| native_extensions::resolve_track_helper(key, ctx))
}

pub fn get_raw(source: &Value, key: &str, default: &str) -> String {
    source.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

pub fn get_raw_with_fallback(source: &Value, album: &Value, key: &str, album_key: &str, default: &str) -> String {
    source.get(key).or_else(|| album.get(album_key)).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

pub fn format_ms(ms: u64) -> String {
    let seconds = (ms / 1000) % 60;
    let minutes = (ms / (1000 * 60)) % 60;
    let hours = ms / (1000 * 60 * 60);
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

pub fn format_date_added(unix: u64) -> String {
    if unix == 0 { return String::new(); }
    Local.timestamp_opt(unix as i64, 0).single().map(|dt| dt.format("%B %d %Y").to_string()).unwrap_or_default()
}

fn calculate_total_discs(tracks: &[Value]) -> u32 {
    let mut discs = HashSet::new();
    for t in tracks {
        if let Some(d) = t.get("DISCNUMBER").and_then(|v| v.as_str()) {
            if let Ok(num) = d.parse::<u64>() {
                discs.insert(num);
            }
        }
    }
    if discs.is_empty() { 1 } else { discs.len() as u32 }
}

fn rel_path(target: &Path, base: &Path) -> String {
    target.strip_prefix(base).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| target.to_string_lossy().to_string())
}

fn format_human_date(yyyy_mm: &str) -> String {
    if yyyy_mm.is_empty() || yyyy_mm == "0000-00" { return "Unknown Date".to_string(); }
    let parts: Vec<&str> = yyyy_mm.split('-').collect();
    let year = parts[0];
    let month_str = parts.get(1).unwrap_or(&"00");
    if *month_str == "00" { return year.to_string(); }
    year.to_string()
}
