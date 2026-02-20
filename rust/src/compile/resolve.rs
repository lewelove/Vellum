use crate::harvest::TrackJson;
use std::collections::HashSet;
use std::path::Path;
use serde_json::{json, Value};
use chrono::{TimeZone, Local};
use image::{DynamicImage, GenericImageView};

pub struct AlbumContext<'a> {
    pub source: &'a Value,
    pub tracks: &'a [Value],
    pub album_root: &'a Path,
    pub library_root: &'a Path,
    pub meta_hash: &'a str,
    pub meta_mtime: u64,
    pub cover_hash: &'a str,
    pub cover_path: Option<&'a str>,
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

// --- Dispatchers ---

pub fn resolve_album_standard(key: &str, ctx: &AlbumContext) -> Option<Value> {
    match key {
        "ALBUM" => Some(json!(get_raw(ctx.source, "ALBUM", "Unknown Album"))),
        "ALBUMARTIST" => Some(json!(get_raw(ctx.source, "ALBUMARTIST", "Unknown Artist"))),
        "GENRE" => Some(json!(resolve_genre(ctx.source))),
        "DATE" => Some(json!(resolve_date_fallback(ctx.source))),
        "ORIGINAL_YYYY_MM" => Some(json!(resolve_original_yyyy_mm(ctx.source))),
        "ORIGINAL_YEAR" => Some(json!(safe_slice(&resolve_original_yyyy_mm(ctx.source), 0, 4))),
        "ORIGINAL_DATE" => Some(json!(format_human_date(&resolve_original_yyyy_mm(ctx.source)))),
        "RELEASE_YYYY_MM" => Some(json!(resolve_release_yyyy_mm(ctx.source))),
        "RELEASE_YEAR" => Some(json!(safe_slice(&resolve_release_yyyy_mm(ctx.source), 0, 4))),
        "RELEASE_DATE" => Some(json!(format_human_date(&resolve_release_yyyy_mm(ctx.source)))),
        "unix_added" => Some(json!(resolve_unix_added(ctx.source))),
        "date_added" => Some(json!(format_date_added(resolve_unix_added(ctx.source)))),
        "album_duration_in_ms" => Some(json!(calculate_total_duration(ctx.tracks))),
        "album_duration_time" => Some(json!(format_ms(calculate_total_duration(ctx.tracks)))),
        "total_tracks" => Some(json!(ctx.tracks.len().to_string())),
        "total_discs" => Some(json!(calculate_total_discs(ctx.tracks).to_string())),
        "album_root_path" => Some(json!(rel_path(ctx.album_root, ctx.library_root))),
        "cover_hash" => Some(json!(ctx.cover_hash)),
        "cover_path" => Some(json!(ctx.cover_path.unwrap_or("default_cover.png"))),
        "cover_mtime" => Some(json!(get_cover_mtime(ctx))),
        "cover_byte_size" => Some(json!(get_cover_size(ctx))),
        "metadata_toml_hash" => Some(json!(ctx.meta_hash)),
        "metadata_toml_mtime" => Some(json!(ctx.meta_mtime)),

        // Perceptual helpers resolved via the loaded image buffer
        "cover_chroma" => resolve_album_helper_cover_chroma(ctx),
        "cover_entropy" => resolve_album_helper_cover_entropy(ctx),
        
        _ => None,
    }
}

pub fn resolve_track_standard(key: &str, ctx: &TrackContext) -> Option<Value> {
    match key {
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
        "encoding" => Some(json!(format_encoding(&ctx.harvest.physics.format))),
        "track_library_path" => Some(json!(rel_path(&ctx.harvest.path, ctx.library_root))),
        "track_path" => Some(json!(rel_path(&ctx.harvest.path, ctx.album_root))),
        _ => None,
    }
}

// --- Perceptual Helpers (Optimized) ---

fn resolve_album_helper_cover_chroma(ctx: &AlbumContext) -> Option<Value> {
    let img = ctx.cover_image?;
    let (width, height) = img.dimensions();
    let total_pixels = (width * height) as f64;
    if total_pixels == 0.0 { return Some(json!(0.0)); }

    let mut sum_rg = 0.0;
    let mut sum_yb = 0.0;
    let mut sq_sum_rg = 0.0;
    let mut sq_sum_yb = 0.0;

    for p in img.pixels() {
        let r = p.2[0] as f64;
        let g = p.2[1] as f64;
        let b = p.2[2] as f64;

        let rg = (r - g).abs();
        let yb = (0.5 * (r + g) - b).abs();

        sum_rg += rg;
        sum_yb += yb;
        sq_sum_rg += rg * rg;
        sq_sum_yb += yb * yb;
    }

    let mean_rg = sum_rg / total_pixels;
    let mean_yb = sum_yb / total_pixels;
    let var_rg = (sq_sum_rg / total_pixels) - (mean_rg * mean_rg);
    let var_yb = (sq_sum_yb / total_pixels) - (mean_yb * mean_yb);
    let std_rg = var_rg.max(0.0).sqrt();
    let std_yb = var_yb.max(0.0).sqrt();

    let std_root = (std_rg.powi(2) + std_yb.powi(2)).sqrt();
    let mean_root = (mean_rg.powi(2) + mean_yb.powi(2)).sqrt();

    Some(json!(std_root + (0.3 * mean_root)))
}

fn resolve_album_helper_cover_entropy(ctx: &AlbumContext) -> Option<Value> {
    let img = ctx.cover_image?;
    let thumb = img.to_luma8();
    
    let mut counts = [0u64; 256];
    for p in thumb.pixels() {
        counts[p[0] as usize] += 1;
    }

    let total = thumb.len() as f64;
    let mut entropy = 0.0;
    for &count in &counts {
        if count > 0 {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
    }
    Some(json!(entropy))
}

// --- Internal Utilities ---

fn safe_slice(s: &str, start: usize, end: usize) -> String {
    if start >= s.len() { return String::new(); }
    let actual_end = std::cmp::min(end, s.len());
    s[start..actual_end].to_string()
}

fn get_raw(source: &Value, key: &str, default: &str) -> String {
    source.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

fn get_raw_with_fallback(source: &Value, album: &Value, key: &str, album_key: &str, default: &str) -> String {
    source.get(key).or_else(|| album.get(album_key)).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

fn resolve_genre(source: &Value) -> Vec<String> {
    let raw = source.get("GENRE").cloned().unwrap_or(json!("Unknown"));
    let mut parts = Vec::new();
    match raw {
        Value::Array(arr) => { for v in arr { if let Some(s) = v.as_str() { parts.push(s.trim().to_string()); } } },
        Value::String(s) => { for part in s.split(';') { let trimmed = part.trim(); if !trimmed.is_empty() { parts.push(trimmed.to_string()); } } },
        _ => {}
    }
    let mut seen = HashSet::new();
    parts.into_iter().filter(|p| seen.insert(p.clone())).collect()
}

fn resolve_date_fallback(source: &Value) -> String {
    source.get("DATE").or_else(|| source.get("YEAR")).or_else(|| source.get("ORIGINALYEAR")).and_then(|v| v.as_str()).unwrap_or("0000").to_string()
}

fn resolve_original_yyyy_mm(source: &Value) -> String {
    if let Some(val) = source.get("ORIGINAL_YYYY_MM").or_else(|| source.get("ORIGINALYEARMONTH")).and_then(|v| v.as_str()) { return val.to_string(); }
    let date = resolve_date_fallback(source);
    if date.len() >= 4 { format!("{}-00", &date[0..4]) } else { "0000-00".to_string() }
}

fn resolve_release_yyyy_mm(source: &Value) -> String {
    if let Some(val) = source.get("RELEASE_YYYY_MM").and_then(|v| v.as_str()) { return val.to_string(); }
    let date = resolve_date_fallback(source);
    if date.len() >= 4 { format!("{}-00", &date[0..4]) } else { "0000-00".to_string() }
}

fn resolve_unix_added(source: &Value) -> u64 {
    let priority_keys = ["UNIX_ADDED_PRIMARY", "UNIX_ADDED_APPLEMUSIC", "UNIX_ADDED_YOUTUBE", "UNIX_ADDED_FOOBAR", "UNIX_ADDED_LOCAL", "UNIXTIMEFOOBAR", "UNIXTIMEAPPLE", "UNIXTIMEYOUTUBE"];
    for key in priority_keys {
        if let Some(val) = source.get(key).and_then(|v| v.as_str()) {
            if let Ok(ts) = val.parse::<u64>() { return ts; }
        }
    }
    0
}

fn format_date_added(unix: u64) -> String {
    if unix == 0 { return String::new(); }
    Local.timestamp_opt(unix as i64, 0).single().map(|dt| dt.format("%B %d %Y").to_string()).unwrap_or_default()
}

fn get_cover_size(ctx: &AlbumContext) -> u64 {
    let Some(p) = ctx.cover_path else { return 0; };
    std::fs::metadata(ctx.album_root.join(p)).map(|m| m.len()).unwrap_or(0)
}

fn get_cover_mtime(ctx: &AlbumContext) -> u64 {
    let Some(p) = ctx.cover_path else { return 0; };
    std::fs::metadata(ctx.album_root.join(p)).and_then(|m| m.modified()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))).map(|d| d.as_secs()).unwrap_or(0)
}

fn calculate_total_duration(tracks: &[Value]) -> u64 {
    tracks.iter().filter_map(|t| t.get("track_duration_in_ms").and_then(|v| v.as_u64())).sum()
}

fn calculate_total_discs(tracks: &[Value]) -> u32 {
    let mut discs = HashSet::new();
    for t in tracks { if let Some(d) = t.get("DISCNUMBER").and_then(|v| v.as_str()) { if let Ok(num) = d.parse::<u64>() { discs.insert(num); } } }
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

fn format_ms(ms: u64) -> String {
    let seconds = (ms / 1000) % 60;
    let minutes = (ms / (1000 * 60)) % 60;
    let hours = ms / (1000 * 60 * 60);
    if hours > 0 { format!("{}:{:02}:{:02}", hours, minutes, seconds) } else { format!("{}:{:02}", minutes, seconds) }
}

fn format_encoding(fmt: &str) -> String {
    match fmt.to_lowercase().as_str() {
        "flac" => "FLAC".to_string(),
        "mp3" => "MP3".to_string(),
        _ => fmt.to_uppercase(),
    }
}
