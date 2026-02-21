use crate::compile::resolve::{self, AlbumContext, TrackContext};
use serde_json::{json, Value};
use std::collections::HashSet;

pub fn resolve_album_tag(key: &str, ctx: &AlbumContext) -> Option<Value> {
    match key {
        "GENRE" => Some(json!(resolve_album_tag_genre(ctx))),
        "DATE" => Some(json!(resolve_album_tag_date(ctx))),
        "ORIGINAL_YYYY_MM" => Some(json!(resolve_album_tag_original_yyyy_mm(ctx))),
        "ORIGINAL_YEAR" => Some(json!(resolve_album_tag_original_yyyy_mm(ctx)[0..4])),
        "RELEASE_YYYY_MM" => Some(json!(resolve_album_tag_release_yyyy_mm(ctx))),
        "RELEASE_YEAR" => Some(json!(resolve_album_tag_release_yyyy_mm(ctx)[0..4])),
        
        // --- Extensions ported to Native ---
        "COUNTRY" => Some(json!(get_str(ctx.source, "COUNTRY"))),
        "LABEL" => Some(json!(get_str(ctx.source, "LABEL"))),
        "CATALOGNUMBER" => Some(json!(get_str(ctx.source, "CATALOGNUMBER"))),
        "MEDIA" => Some(json!(get_str(ctx.source, "MEDIA"))),
        
        "DISCOGS_URL" => Some(json!(get_str(ctx.source, "DISCOGS_URL"))),
        "MUSICBRAINZ_URL" => Some(json!(get_str(ctx.source, "MUSICBRAINZ_URL"))),
        
        "UNIX_ADDED_LOCAL" => Some(json!(resolve_album_tag_unix_added_local(ctx))),
        "UNIX_ADDED_FOOBAR" => Some(json!(resolve_album_tag_unix_added_foobar(ctx))),
        "UNIX_ADDED_APPLEMUSIC" => Some(json!(resolve_album_tag_unix_added_applemusic(ctx))),
        "UNIX_ADDED_YOUTUBE" => Some(json!(resolve_album_tag_unix_added_youtube(ctx))),
        "UNIX_ADDED_PRIMARY" => Some(json!(get_str(ctx.source, "UNIX_ADDED_PRIMARY"))),
        
        "CUSTOM_ID" => Some(json!(get_str(ctx.source, "CUSTOM_ID"))),
        "CUSTOM_ALBUMARTIST" => Some(json!(resolve_album_tag_custom_albumartist(ctx))),
        "CUSTOM_STRING" => Some(json!(resolve_album_tag_custom_string(ctx))),
        "OLD_COMMENT" => Some(json!(get_str(ctx.source, "OLD_COMMENT"))),
        "COMMENT" => Some(json!(resolve_album_tag_comment(ctx))),
        
        "REPLAYGAIN_ALBUM_GAIN" => Some(json!(get_str(ctx.source, "REPLAYGAIN_ALBUM_GAIN"))),
        "REPLAYGAIN_ALBUM_PEAK" => Some(json!(get_str(ctx.source, "REPLAYGAIN_ALBUM_PEAK"))),
        
        "MUSICBRAINZ_ALBUMID" => Some(json!(get_str(ctx.source, "MUSICBRAINZ_ALBUMID"))),
        "MUSICBRAINZ_ALBUMARTISTID" => Some(json!(get_str(ctx.source, "MUSICBRAINZ_ALBUMARTISTID"))),
        "MUSICBRAINZ_RELEASEGROUPID" => Some(json!(get_str(ctx.source, "MUSICBRAINZ_RELEASEGROUPID"))),
        
        _ => None,
    }
}

pub fn resolve_album_helper(key: &str, ctx: &AlbumContext) -> Option<Value> {
    match key {
        "album_duration_in_ms" => Some(json!(resolve_album_helper_duration_ms(ctx))),
        "album_duration_time" => Some(json!(resolve::format_ms(resolve_album_helper_duration_ms(ctx)))),
        "date_added" => Some(json!(resolve::format_date_added(resolve_album_helper_unix_added(ctx)))),
        "unix_added" => Some(json!(resolve_album_helper_unix_added(ctx))),
        "cover_chroma" => resolve_album_helper_cover_chroma(ctx),
        "cover_entropy" => resolve_album_helper_cover_entropy(ctx),
        _ => None,
    }
}

pub fn resolve_track_tag(key: &str, ctx: &TrackContext) -> Option<Value> {
    match key {
        "ACCURIPID" => Some(json!(get_str(ctx.source, "ACCURIPID"))),
        "CTDBID" => Some(json!(get_str(ctx.source, "CTDBID"))),
        "DISCID" => Some(json!(get_str(ctx.source, "DISCID"))),
        
        "REPLAYGAIN_TRACK_GAIN" => Some(json!(get_str(ctx.source, "REPLAYGAIN_TRACK_GAIN"))),
        "REPLAYGAIN_TRACK_PEAK" => Some(json!(get_str(ctx.source, "REPLAYGAIN_TRACK_PEAK"))),
        
        "MUSICBRAINZ_TRACKID" => Some(json!(get_str(ctx.source, "MUSICBRAINZ_TRACKID"))),
        "MUSICBRAINZ_RELEASETRACKID" => Some(json!(get_str(ctx.source, "MUSICBRAINZ_RELEASETRACKID"))),
        "MUSICBRAINZ_ARTISTID" => Some(json!(get_str(ctx.source, "MUSICBRAINZ_ARTISTID"))),
        
        _ => None,
    }
}

pub fn resolve_track_helper(key: &str, ctx: &TrackContext) -> Option<Value> {
    match key {
        "encoding" => Some(json!(resolve_track_helper_encoding(ctx))),
        _ => None,
    }
}

// --- Helpers ---

fn get_str(source: &Value, key: &str) -> String {
    source.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn resolve_album_tag_genre(ctx: &AlbumContext) -> Vec<String> {
    let raw = ctx.source.get("GENRE").cloned().unwrap_or(json!("Unknown"));
    let mut parts = Vec::new();
    match raw {
        Value::Array(arr) => {
            for v in arr {
                if let Some(s) = v.as_str() {
                    parts.push(s.trim().to_string());
                }
            }
        }
        Value::String(s) => {
            for part in s.split(';') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
            }
        }
        _ => {}
    }
    let mut seen = HashSet::new();
    parts.into_iter().filter(|p| seen.insert(p.clone())).collect()
}

fn resolve_album_tag_date(ctx: &AlbumContext) -> String {
    ctx.source.get("DATE")
        .or_else(|| ctx.source.get("YEAR"))
        .or_else(|| ctx.source.get("ORIGINALYEAR"))
        .and_then(|v| v.as_str())
        .unwrap_or("0000")
        .to_string()
}

fn resolve_album_tag_original_yyyy_mm(ctx: &AlbumContext) -> String {
    if let Some(val) = ctx.source.get("ORIGINAL_YYYY_MM")
        .or_else(|| ctx.source.get("ORIGINALYEARMONTH"))
        .and_then(|v| v.as_str()) {
        return val.to_string();
    }
    let date = resolve_album_tag_date(ctx);
    if date.len() >= 4 {
        format!("{}-00", &date[0..4])
    } else {
        "0000-00".to_string()
    }
}

fn resolve_album_tag_release_yyyy_mm(ctx: &AlbumContext) -> String {
    if let Some(val) = ctx.source.get("RELEASE_YYYY_MM").and_then(|v| v.as_str()) {
        return val.to_string();
    }
    let date = resolve_album_tag_date(ctx);
    if date.len() >= 4 {
        format!("{}-00", &date[0..4])
    } else {
        "0000-00".to_string()
    }
}

fn resolve_album_tag_custom_albumartist(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("CUSTOM_ALBUMARTIST").and_then(|s| s.as_str()) { return v.to_string(); }
    if let Some(v) = ctx.source.get("ARTISTARTIST").and_then(|s| s.as_str()) { return v.to_string(); }
    if let Some(v) = ctx.source.get("ALBUMARTIST").and_then(|s| s.as_str()) { return v.to_string(); }
    "Unknown".to_string()
}

fn resolve_album_tag_custom_string(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("CUSTOM_STRING").and_then(|s| s.as_str()) { return v.to_string(); }
    if let Some(v) = ctx.source.get("CUSTOMSTRING").and_then(|s| s.as_str()) { return v.to_string(); }
    "".to_string()
}

fn resolve_album_tag_comment(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("COMMENT").and_then(|s| s.as_str()) {
        if !v.is_empty() { return v.to_string(); }
    }

    let country = get_str(ctx.source, "COUNTRY");
    let label = get_str(ctx.source, "LABEL");
    let cat_no = get_str(ctx.source, "CATALOGNUMBER");

    if country.is_empty() && label.is_empty() && cat_no.is_empty() {
        return "".to_string();
    }

    let yyyy_mm = resolve_album_tag_release_yyyy_mm(ctx);
    let year = if yyyy_mm.len() >= 4 { &yyyy_mm[0..4] } else { "" };
    
    let parts = [year, &country, &label, &cat_no];
    parts.iter()
        .filter(|s| !s.is_empty())
        .map(|s| *s)
        .collect::<Vec<&str>>()
        .join(" ")
}

fn resolve_album_tag_unix_added_local(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("UNIX_ADDED_LOCAL").and_then(|s| s.as_str()) { return v.to_string(); }
    if let Some(v) = ctx.source.get("UNIX_ADDED_PRIMARY").and_then(|s| s.as_str()) { return v.to_string(); }
    "".to_string()
}

fn resolve_album_tag_unix_added_foobar(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("UNIX_ADDED_FOOBAR").and_then(|s| s.as_str()) { return v.to_string(); }
    if let Some(v) = ctx.source.get("UNIXTIMEFOOBAR").and_then(|s| s.as_str()) { return v.to_string(); }
    "".to_string()
}

fn resolve_album_tag_unix_added_applemusic(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("UNIX_ADDED_APPLEMUSIC").and_then(|s| s.as_str()) { return v.to_string(); }
    if let Some(v) = ctx.source.get("UNIXTIMEAPPLE").and_then(|s| s.as_str()) { return v.to_string(); }
    "".to_string()
}

fn resolve_album_tag_unix_added_youtube(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("UNIX_ADDED_YOUTUBE").and_then(|s| s.as_str()) { return v.to_string(); }
    if let Some(v) = ctx.source.get("UNIXTIMEYOUTUBE").and_then(|s| s.as_str()) { return v.to_string(); }
    "".to_string()
}

fn resolve_album_helper_duration_ms(ctx: &AlbumContext) -> u64 {
    ctx.tracks.iter().filter_map(|t| t.get("track_duration_in_ms").and_then(|v| v.as_u64())).sum()
}

fn resolve_album_helper_unix_added(ctx: &AlbumContext) -> u64 {
    let priority_keys = [
        "UNIX_ADDED_PRIMARY",
        "UNIX_ADDED_APPLEMUSIC",
        "UNIX_ADDED_YOUTUBE",
        "UNIX_ADDED_FOOBAR",
        "UNIX_ADDED_LOCAL",
        "UNIXTIMEFOOBAR",
        "UNIXTIMEAPPLE",
        "UNIXTIMEYOUTUBE",
    ];
    for key in priority_keys {
        if let Some(val) = ctx.source.get(key).and_then(|v| v.as_str()) {
            if let Ok(ts) = val.parse::<u64>() {
                return ts;
            }
        }
    }
    0
}

fn resolve_album_helper_cover_chroma(ctx: &AlbumContext) -> Option<Value> {
    use image::GenericImageView;
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

fn resolve_track_helper_encoding(ctx: &TrackContext) -> String {
    match ctx.harvest.physics.format.to_lowercase().as_str() {
        "flac" => "FLAC".to_string(),
        "mp3" => "MP3".to_string(),
        _ => ctx.harvest.physics.format.to_uppercase(),
    }
}
