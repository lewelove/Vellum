use crate::compile::builder::context::AlbumContext;
use crate::compile::resolvers::standard;
use image::GenericImageView;
use image::imageops::FilterType;
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
    let img = ctx.cover_image?;
    
    let discovery_img = img.resize_exact(64, 64, FilterType::Nearest);
    let discovery_pixels: Vec<Lab> = discovery_img
        .pixels()
        .map(|(_, _, p)| {
            let srgb = Srgb::new(
                p[0] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[2] as f32 / 255.0,
            );
            Lab::from_color(srgb)
        })
        .collect();

    let sample_size = 256;
    let step = (discovery_pixels.len() / sample_size).max(1);
    let sample: Vec<Lab> = discovery_pixels.iter().step_by(step).take(sample_size).copied().collect();
    
    let mut distances = Vec::with_capacity(sample.len() * (sample.len() - 1) / 2);
    for i in 0..sample.len() {
        for j in (i + 1)..sample.len() {
            let p1 = sample[i];
            let p2 = sample[j];
            let dl = p1.l - p2.l;
            let da = p1.a - p2.a;
            let db = p1.b - p2.b;
            distances.push((dl * dl + da * da + db * db).sqrt());
        }
    }
    
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let quantile_idx = (distances.len() as f32 * 0.15) as usize;
    let estimated_bandwidth = if distances.is_empty() {
        10.0
    } else {
        distances[quantile_idx].clamp(4.0, 20.0)
    };

    let bandwidth_sq = estimated_bandwidth * estimated_bandwidth;
    let convergence_sq = 0.25_f32;
    let max_iter = 15;

    let mut raw_modes = Vec::with_capacity(discovery_pixels.len());

    for &seed in &discovery_pixels {
        let mut current = seed;
        for _ in 0..max_iter {
            let mut sum_l = 0.0;
            let mut sum_a = 0.0;
            let mut sum_b = 0.0;
            let mut count = 0.0;

            for &p in &discovery_pixels {
                let dl = p.l - current.l;
                let da = p.a - current.a;
                let db = p.b - current.b;
                let dist_sq = dl * dl + da * da + db * db;

                if dist_sq <= bandwidth_sq {
                    sum_l += p.l;
                    sum_a += p.a;
                    sum_b += p.b;
                    count += 1.0;
                }
            }

            if count > 0.0 {
                let new_l = sum_l / count;
                let new_a = sum_a / count;
                let new_b = sum_b / count;

                let dl = new_l - current.l;
                let da = new_a - current.a;
                let db = new_b - current.b;
                let shift_sq = dl * dl + da * da + db * db;

                current = Lab::new(new_l, new_a, new_b);

                if shift_sq < convergence_sq {
                    break;
                }
            } else {
                break;
            }
        }
        raw_modes.push(current);
    }

    let mut merged_centers: Vec<Lab> = Vec::new();
    let merge_threshold_sq = 144.0_f32;

    for mode in raw_modes {
        let mut is_merged = false;
        for center in &merged_centers {
            let dl = center.l - mode.l;
            let da = center.a - mode.a;
            let db = center.b - mode.b;
            if dl * dl + da * da + db * db < merge_threshold_sq {
                is_merged = true;
                break;
            }
        }
        if !is_merged {
            merged_centers.push(mode);
        }
    }
    
    let ratio_img = img.resize_exact(256, 256, FilterType::Nearest);
    let mut physical_counts = vec![0_usize; merged_centers.len()];

    for (_, _, p) in ratio_img.pixels() {
        let srgb = Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        );
        let pixel_lab = Lab::from_color(srgb);

        let mut best_dist = f32::MAX;
        let mut best_idx = 0;

        for (idx, center_lab) in merged_centers.iter().enumerate() {
            let dl = pixel_lab.l - center_lab.l;
            let da = pixel_lab.a - center_lab.a;
            let db = pixel_lab.b - center_lab.b;
            let dist_sq = dl * dl + da * da + db * db;

            if dist_sq < best_dist {
                best_dist = dist_sq;
                best_idx = idx;
            }
        }
        physical_counts[best_idx] += 1;
    }

    let total_samples = (256 * 256) as f64;

    let mut representation: Vec<(f64, Lab)> = merged_centers
        .into_iter()
        .enumerate()
        .map(|(i, lab)| {
            let ratio = (physical_counts[i] as f64) / total_samples;
            (ratio, lab)
        })
        .collect();

    representation.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let palette_data: Vec<Value> = representation
        .into_iter()
        .map(|(ratio, lab)| {
            let srgb = Srgb::from_color(lab);
            let hex = format!(
                "#{:02X}{:02X}{:02X}",
                (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8
            );

            json!([hex, format!("{ratio:.4}")])
        })
        .collect();

    Some(json!(palette_data))
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
