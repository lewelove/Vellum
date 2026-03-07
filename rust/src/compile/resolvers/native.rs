use crate::compile::builder::context::AlbumContext;
use crate::compile::resolvers::standard;
use image::GenericImageView;
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
            && let Ok(ts) = val.parse::<u64>()
        {
            return ts;
        }
    }
    0
}

pub fn resolve_custom_albumartist(ctx: &AlbumContext) -> String {
    let keys = ["custom_albumartist", "artistartist", "albumartist"];
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
    let img = ctx.cover_image?;
    let k = 6;
    let max_iters = 10;

    let small = img.resize_exact(64, 64, image::imageops::FilterType::Nearest);

    let pixels: Vec<[f32; 3]> = small
        .pixels()
        .map(|(_, _, p)| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();

    if pixels.is_empty() {
        return None;
    }

    let mut centroids = Vec::with_capacity(k);
    let step = pixels.len() / k.max(1);
    for i in 0..k {
        centroids.push(pixels[(i * step).min(pixels.len() - 1)]);
    }

    let mut assignments = vec![0; pixels.len()];
    let mut counts = vec![0; k];

    for _ in 0..max_iters {
        let mut changed = false;

        for (i, p) in pixels.iter().enumerate() {
            let mut min_dist = f32::MAX;
            let mut best_k = 0;

            for (j, c) in centroids.iter().enumerate() {
                let dr = p[0] - c[0];
                let dg = p[1] - c[1];
                let db = p[2] - c[2];
                let dist_sq = dr.mul_add(dr, dg.mul_add(dg, db * db));

                if dist_sq < min_dist {
                    min_dist = dist_sq;
                    best_k = j;
                }
            }

            if assignments[i] != best_k {
                assignments[i] = best_k;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        let mut new_centroids = vec![[0.0; 3]; k];
        counts.fill(0);

        for (i, p) in pixels.iter().enumerate() {
            let cluster = assignments[i];
            new_centroids[cluster][0] += p[0];
            new_centroids[cluster][1] += p[1];
            new_centroids[cluster][2] += p[2];
            counts[cluster] += 1;
        }

        for j in 0..k {
            if counts[j] > 0 {
                centroids[j][0] = new_centroids[j][0] / counts[j] as f32;
                centroids[j][1] = new_centroids[j][1] / counts[j] as f32;
                centroids[j][2] = new_centroids[j][2] / counts[j] as f32;
            }
        }
    }

    let mut cluster_data: Vec<(usize, [f32; 3])> = counts
        .into_iter()
        .zip(centroids)
        .collect();

    cluster_data.sort_by(|a, b| b.0.cmp(&a.0));

    let hex_palette: Vec<String> = cluster_data
        .into_iter()
        .filter(|(count, _)| *count > 0)
        .map(|(_, c)| format!("#{:02X}{:02X}{:02X}", c[0] as u8, c[1] as u8, c[2] as u8))
        .collect();

    Some(json!(hex_palette))
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
    
    let parts = [
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
    let folders = ["lyrics", "Lyrics"];
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
