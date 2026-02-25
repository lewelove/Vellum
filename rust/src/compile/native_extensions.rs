use crate::compile::resolve::{AlbumContext, TrackContext};
use serde_json::{json, Value};
use std::collections::HashSet;

pub fn resolve_album_key(key: &str, ctx: &AlbumContext) -> Option<Value> {
    match key {
        "custom_albumartist" => Some(json!(resolve_custom_albumartist(ctx))),
        "cover_chroma" => resolve_cover_chroma(ctx),
        "cover_entropy" => resolve_cover_entropy(ctx),
        _ => None,
    }
}

pub fn resolve_track_key(_key: &str, _ctx: &TrackContext) -> Option<Value> {
    match _key {
        _ => None,
    }
}

fn get_str(source: &Value, key: &str) -> String {
    source.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

pub fn resolve_genre(ctx: &AlbumContext) -> Vec<String> {
    let raw = ctx.source.get("genre").cloned().unwrap_or(json!(null));
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
            for p in s.split(';') {
                let t = p.trim();
                if !t.is_empty() {
                    parts.push(t.to_string());
                }
            }
        }
        _ => {}
    }

    if parts.is_empty() {
        parts.push("Unknown".to_string());
    }

    let mut seen = HashSet::new();
    parts.into_iter().filter(|p| seen.insert(p.clone())).collect()
}

pub fn resolve_date(ctx: &AlbumContext) -> String {
    ctx.source.get("date")
        .or_else(|| ctx.source.get("year"))
        .or_else(|| ctx.source.get("originalyear"))
        .and_then(|v| v.as_str())
        .unwrap_or("0000")
        .to_string()
}

pub fn resolve_original_yyyy_mm(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("original_yyyy_mm").or_else(|| ctx.source.get("originalyearmonth")).and_then(|v| v.as_str()) {
        return v.to_string();
    }
    let d = resolve_date(ctx);
    if d.len() >= 4 { format!("{}-00", &d[0..4]) } else { "0000-00".to_string() }
}

pub fn resolve_release_yyyy_mm(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("release_yyyy_mm").and_then(|v| v.as_str()) { return v.to_string(); }
    let d = resolve_date(ctx);
    if d.len() >= 4 { format!("{}-00", &d[0..4]) } else { "0000-00".to_string() }
}

fn resolve_custom_albumartist(ctx: &AlbumContext) -> String {
    let keys = ["custom_albumartist", "artistartist", "albumartist"];
    for k in keys { if let Some(v) = ctx.source.get(k).and_then(|s| s.as_str()) { return v.to_string(); } }
    "Unknown".to_string()
}

pub fn resolve_comment(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("comment").and_then(|s| s.as_str()) {
        if !v.is_empty() { return v.to_string(); }
    }
    let country = get_str(ctx.source, "country");
    let label = get_str(ctx.source, "label");
    let cat = get_str(ctx.source, "catalognumber");
    if country.is_empty() && label.is_empty() && cat.is_empty() { return "".to_string(); }
    let yyyy_mm = resolve_release_yyyy_mm(ctx);
    let year = if yyyy_mm.len() >= 4 { &yyyy_mm[0..4] } else { "" };
    [year, &country, &label, &cat].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" ")
}

fn resolve_cover_chroma(ctx: &AlbumContext) -> Option<Value> {
    use image::GenericImageView;
    let img = ctx.cover_image?;
    let (w, h) = img.dimensions();
    let total = (w * h) as f64;
    if total == 0.0 { return Some(json!(0.0)); }
    let mut s_rg = 0.0;
    let mut s_yb = 0.0;
    let mut sq_rg = 0.0;
    let mut sq_yb = 0.0;
    for p in img.pixels() {
        let r = p.2[0] as f64;
        let g = p.2[1] as f64;
        let b = p.2[2] as f64;
        let rg = (r - g).abs();
        let yb = (0.5 * (r + g) - b).abs();
        s_rg += rg; s_yb += yb;
        sq_rg += rg * rg; sq_yb += yb * yb;
    }
    let m_rg = s_rg / total;
    let m_yb = s_yb / total;
    let v_rg = (sq_rg / total) - (m_rg * m_rg);
    let v_yb = (sq_yb / total) - (m_yb * m_yb);
    let std_root = (v_rg.max(0.0) + v_yb.max(0.0)).sqrt();
    let mean_root = (m_rg.powi(2) + m_yb.powi(2)).sqrt();
    Some(json!(std_root + (0.3 * mean_root)))
}

fn resolve_cover_entropy(ctx: &AlbumContext) -> Option<Value> {
    let img = ctx.cover_image?;
    let thumb = img.to_luma8();
    let mut counts = [0u64; 256];
    for p in thumb.pixels() { counts[p[0] as usize] += 1; }
    let total = thumb.len() as f64;
    let mut ent = 0.0;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / total;
            ent -= p * p.log2();
        }
    }
    Some(json!(ent))
}
