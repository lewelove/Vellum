use crate::compile::builder::context::AlbumContext;
use crate::compile::resolvers::standard;
use image::GenericImageView;
use image::imageops::FilterType;
use kmeans_colors::get_kmeans_hamerly;
use palette::{FromColor, Lab, Srgb};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::path::Path;

pub fn resolve_date(ctx: &AlbumContext) -> String {
    ctx.source
        .get("date")
        .or_else(|| ctx.source.get("year"))
        .or_else(|| ctx.source.get("originalyear"))
        .and_then(Value::as_str)
        .unwrap_or("0000")
        .to_string()
}

pub fn resolve_yyyy_mm(ctx: &AlbumContext, key: &str) -> String {
    if let Some(v) = ctx.source.get(key).and_then(Value::as_str) {
        return v.to_string();
    }
    let d = resolve_date(ctx);
    if d.len() >= 4 {
        format!("{}-00", &d[0..4])
    } else {
        "0000-00".to_string()
    }
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
    parts
        .into_iter()
        .filter(|p| seen.insert(p.clone()))
        .collect()
}

pub fn calculate_total_discs(tracks: &[Value]) -> u32 {
    let mut discs = HashSet::new();
    for t in tracks {
        let val = match t.get("DISCNUMBER") {
            Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
            Some(Value::String(s)) => s
                .split('/')
                .next()
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0),
            _ => 0,
        };
        if val > 0 {
            discs.insert(val);
        }
    }
    if discs.is_empty() {
        1
    } else {
        u32::try_from(discs.len()).unwrap_or(u32::MAX)
    }
}

pub fn resolve_album_info_unix_added(ctx: &AlbumContext) -> u64 {
    let keys =[
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
            && let Ok(ts) = val.parse::<u64>()
        {
            return ts;
        }
    }
    0
}

pub fn resolve_custom_albumartist(ctx: &AlbumContext) -> String {
    let keys =[
        "custom_albumartist",
        "artistartist",
        "albumartist",
    ];
    for k in keys {
        if let Some(v) = ctx.source.get(k).and_then(Value::as_str) {
            return v.to_string();
        }
    }
    "Unknown".to_string()
}

pub fn rel_path(target: &Path, base: &Path) -> String {
    target.strip_prefix(base).map_or_else(
        |_| target.to_string_lossy().to_string(),
        |p| p.to_string_lossy().to_string(),
    )
}

pub fn resolve_cover_chroma(ctx: &AlbumContext) -> Option<Value> {
    let img = ctx.cover_image?;
    let (width, height) = img.dimensions();
    let total = f64::from(width * height);
    if total == 0.0 {
        return Some(json!(0.0));
    }

    let mut sum_rg = 0.0;
    let mut sum_yb = 0.0;
    let mut sum_sq_rg = 0.0;
    let mut sum_sq_yb = 0.0;

    for p in img.pixels() {
        let r = f64::from(p.2[0]);
        let g = f64::from(p.2[1]);
        let b = f64::from(p.2[2]);
        let rg = (r - g).abs();
        let yb = (0.5f64.mul_add(r + g, -b)).abs();
        sum_rg += rg;
        sum_yb += yb;
        sum_sq_rg += rg * rg;
        sum_sq_yb += yb * yb;
    }

    let m_rg = sum_rg / total;
    let m_yb = sum_yb / total;
    let v_rg = m_rg.mul_add(-m_rg, sum_sq_rg / total);
    let v_yb = m_yb.mul_add(-m_yb, sum_sq_yb / total);
    let std_root = (v_rg.max(0.0) + v_yb.max(0.0)).sqrt();
    let mean_root = m_rg.hypot(m_yb);
    Some(json!(0.3f64.mul_add(mean_root, std_root)))
}

pub fn resolve_cover_entropy(ctx: &AlbumContext) -> Option<Value> {
    let img = ctx.cover_image?;
    let gray = img.grayscale();
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    gray.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
    Some(json!(buf.len()))
}

pub fn resolve_cover_palette(ctx: &AlbumContext) -> Option<Value> {
    let cover_path = ctx.cover_path?;
    let img = image::open(ctx.album_root.join(cover_path)).ok()?;

    // 1. Downsample to 256p using Nearest Neighbor to preserve raw color clusters
    let img_small = img.resize_exact(256, 256, FilterType::Nearest);
    let mut pixels: Vec<Lab> = Vec::with_capacity(256 * 256);

    for (_, _, p) in img_small.pixels() {
        pixels.push(Lab::from_color(Srgb::new(
            f32::from(p[0]) / 255.0,
            f32::from(p[1]) / 255.0,
            f32::from(p[2]) / 255.0,
        )));
    }

    // 2. Run K-Means with k=24
    let k = 24;
    let max_iter = 20;
    let convergence = 0.005;
    let result = get_kmeans_hamerly(k, max_iter, convergence, false, &pixels, 42);
    let total_px = 65536.0_f32; // 256 * 256

    // 3. Noise Cleaning: Identify clusters with very low representation
    // Threshold is ~32 pixels (0.0005)
    let discard_threshold = 0.0005_f32;
    let mut counts = vec![0_usize; k];
    for &idx in &result.indices {
        counts[idx as usize] += 1;
    }

    let mut keep_indices = Vec::new();
    let mut discard_indices = Vec::new();

    for i in 0..k {
        let ratio = counts[i] as f32 / total_px;
        if ratio >= discard_threshold {
            keep_indices.push(i);
        } else {
            discard_indices.push(i);
        }
    }

    // Fallback: Ensure at least one cluster exists
    if keep_indices.is_empty() {
        let max_idx = counts.iter().enumerate()
            .max_by_key(|&(_, count)| count)
            .map(|(i, _)| i)
            .unwrap_or(0);
        keep_indices.push(max_idx);
        discard_indices.retain(|&i| i != max_idx);
    }

    // 4. Remapping: Transfer mass of discarded clusters to their nearest neighbors
    let mut final_counts = counts.clone();
    for &d_idx in &discard_indices {
        let d_lab = result.centroids[d_idx];
        let mut best_target = keep_indices[0];
        let mut min_dist_sq = f32::MAX;

        for &k_idx in &keep_indices {
            let k_lab = result.centroids[k_idx];
            let dist_sq = (d_lab.l - k_lab.l).powi(2) 
                        + (d_lab.a - k_lab.a).powi(2) 
                        + (d_lab.b - k_lab.b).powi(2);
            
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_target = k_idx;
            }
        }
        final_counts[best_target] += counts[d_idx];
    }

    // 5. Generate and Sort Palette
    let mut palette: Vec<(Lab, f32)> = keep_indices.iter()
        .map(|&i| (result.centroids[i], final_counts[i] as f32 / total_px))
        .collect();
    
    palette.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let palette_json: Vec<Value> = palette.iter()
        .map(|(lab, ratio)| {
            let srgb = Srgb::from_color(*lab);
            let hex = format!("#{:02X}{:02X}{:02X}", 
                (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8
            );
            json!([hex, format!("{ratio:.4}")])
        })
        .collect();

    Some(json!(palette_json))
}

pub fn resolve_comment(ctx: &AlbumContext) -> String {
    if let Some(v) = ctx.source.get("comment").and_then(Value::as_str)
        && !v.is_empty()
    {
        return v.to_string();
    }

    let country = standard::get_raw(ctx.source, "country", "");
    let label = standard::get_raw(ctx.source, "label", "");
    let cat = standard::get_raw(ctx.source, "catalognumber", "");
    if country.is_empty() && label.is_empty() && cat.is_empty() {
        return String::new();
    }
    let yyyy_mm = resolve_yyyy_mm(ctx, "release_yyyy_mm");
    let year = if yyyy_mm.len() >= 4 {
        &yyyy_mm[0..4]
    } else {
        ""
    };
    
    let parts =[
        year,
        &country,
        &label,
        &cat,
    ];

    parts
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn resolve_lyrics_path(
    album_root: &Path,
    track_num: u32,
    disc_num: u32,
    total_discs: u32,
) -> Option<String> {
    let folders =[
        "lyrics",
        "Lyrics",
    ];
    let mut candidates = Vec::new();

    for folder in folders {
        let dir = album_root.join(folder);
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(ext) = path
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_lowercase)
            else {
                continue;
            };
            if ext != "lrc" && ext != "txt" {
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            let is_match = if total_discs > 1 {
                name.find('.').is_some_and(|dot_idx| {
                    let disc_part = &name[..dot_idx];
                    let remaining = &name[dot_idx + 1..];
                    let track_part = remaining
                        .chars()
                        .take_while(char::is_ascii_digit)
                        .collect::<String>();

                    let d_match = disc_part.parse::<u32>().is_ok_and(|d| d == disc_num);
                    let t_match = track_part.parse::<u32>().is_ok_and(|t| t == track_num);
                    d_match && t_match
                })
            } else {
                let track_part = name
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>();
                track_part.parse::<u32>().is_ok_and(|t| t == track_num)
            };

            if is_match {
                candidates.push((rel_path(&path, album_root), ext));
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    if let Some(lrc) = candidates.iter().find(|(_, ext)| ext == "lrc") {
        return Some(lrc.0.clone());
    }

    candidates.first().map(|(path, _)| path.clone())
}
