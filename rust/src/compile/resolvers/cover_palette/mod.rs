pub mod kmeans;
pub mod kmeans_filtered;
pub mod mcu_material;
pub mod mean_shift;
pub mod palette_extract_crate;

use crate::compile::builder::context::AlbumContext;
use image::imageops::FilterType;
use mcu_hct::Hct;
use palette::{FromColor, Oklab, Oklch, Srgb};
use serde_json::{Value, json};

fn parse_hex(hex: &str) -> Option<Srgb> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
}

pub fn resolve(ctx: &AlbumContext, args: &str) -> Option<Value> {
    let cover_path = ctx.cover_path?;
    let img = image::open(ctx.album_root.join(cover_path)).ok()?;

    let sample_dim = args.split(',')
        .find(|s| s.trim().starts_with("dim="))
        .and_then(|s| s.trim().strip_prefix("dim="))
        .and_then(|val| val.parse::<u32>().ok())
        .unwrap_or(512);

    let img_to_process = if sample_dim == 0 {
        img
    } else {
        img.resize_exact(sample_dim, sample_dim, FilterType::Nearest)
    };

    let mut candidate_colors = Vec::new();
    let mut manually_provided = false;
    
    if let Some(arr) = ctx.source.get("cover_palette").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                if let Some(srgb) = parse_hex(s) {
                    candidate_colors.push(srgb);
                }
            }
        }
        if !candidate_colors.is_empty() {
            manually_provided = true;
        }
    }

    if candidate_colors.is_empty() {
        let algo_type = args.split(',')
            .find(|s| s.trim().starts_with("type="))
            .map(|s| s.trim().strip_prefix("type=").unwrap())
            .unwrap_or("kmeans");

        candidate_colors = match algo_type {
            "msc" => mean_shift::extract(&img_to_process, args),
            "material" => mcu_material::extract(&img_to_process, args),
            "kmeans_filtered" => kmeans_filtered::extract(&img_to_process, args),
            "mmcq" => palette_extract_crate::extract(&img_to_process, args),
            "kmeans" | _ => kmeans::extract(&img_to_process, args),
        };
    }

    if candidate_colors.is_empty() {
        return None;
    }

    let oklab_centers: Vec<Oklab> = candidate_colors.iter().map(|&c| Oklab::from_color(c)).collect();
    let mut counts = vec![0usize; oklab_centers.len()];

    for p in img_to_process.to_rgb8().pixels() {
        let pixel_oklab = Oklab::from_color(Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ));
        let mut best_idx = 0;
        let mut min_dist_sq = f32::MAX;
        for (i, center) in oklab_centers.iter().enumerate() {
            let dist_sq = (pixel_oklab.l - center.l).powi(2)
                        + (pixel_oklab.a - center.a).powi(2)
                        + (pixel_oklab.b - center.b).powi(2);
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_idx = i;
            }
        }
        counts[best_idx] += 1;
    }

    let total_pixels = counts.iter().sum::<usize>() as f32;
    let mut palette: Vec<(Srgb, f32)> = candidate_colors.into_iter().zip(counts.into_iter()).filter_map(|(color, count)| {
        let ratio = if total_pixels > 0.0 {
            count as f32 / total_pixels
        } else {
            0.0
        };
        
        if manually_provided || ratio > 0.0 {
            Some((color, ratio))
        } else {
            None
        }
    }).collect();

    let threshold = args.split(',')
        .find(|s| s.trim().starts_with("t="))
        .and_then(|s| s.trim().strip_prefix("t="))
        .and_then(|val| val.parse::<f32>().ok())
        .unwrap_or(0.001);

    if !manually_provided {
        palette.retain(|&(_, ratio)| ratio >= threshold);
    }

    let final_total: f32 = palette.iter().map(|(_, r)| r).sum();
    if final_total > 0.0 {
        for item in &mut palette {
            item.1 /= final_total;
        }
    } else if manually_provided && !palette.is_empty() {
        let even = 1.0 / palette.len() as f32;
        for item in &mut palette {
            item.1 = even;
        }
    }

    let sort_type = if manually_provided {
        "original"
    } else {
        args.split(',')
            .find(|s| s.trim().starts_with("sort="))
            .and_then(|s| s.trim().strip_prefix("sort="))
            .unwrap_or("ratio")
    };

    match sort_type {
        "L" => palette.sort_by(|a, b| {
            let l_a = Oklch::from_color(a.0).l;
            let l_b = Oklch::from_color(b.0).l;
            l_b.partial_cmp(&l_a).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "C" => palette.sort_by(|a, b| {
            let c_a = Oklch::from_color(a.0).chroma;
            let c_b = Oklch::from_color(b.0).chroma;
            c_b.partial_cmp(&c_a).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "H" => palette.sort_by(|a, b| {
            let h_a = Oklch::from_color(a.0).hue.into_raw_degrees();
            let h_b = Oklch::from_color(b.0).hue.into_raw_degrees();
            h_a.partial_cmp(&h_b).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "original" => {},
        "ratio" | _ => palette.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)),
    }

    let palette_json: Vec<Value> = palette.into_iter()
        .map(|(srgb, ratio)| {
            let r_u8 = (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8;
            let g_u8 = (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8;
            let b_u8 = (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8;

            let hex = format!("#{:02X}{:02X}{:02X}", r_u8, g_u8, b_u8);

            let oklch = Oklch::from_color(srgb);
            let l_pct = oklch.l * 100.0;
            let c = oklch.chroma;
            let h = oklch.hue.into_raw_degrees();
            let oklch_str = format!("oklch({:.2}% {:.3} {:.2})", l_pct, c, h);

            let argb = 0xFF00_0000 | ((r_u8 as u32) << 16) | ((g_u8 as u32) << 8) | (b_u8 as u32);
            let hct = Hct::from_int(argb);
            let hct_str = format!("hct({:.2} {:.2} {:.2})", hct.hue(), hct.chroma(), hct.tone());

            json!([hex, oklch_str, hct_str, format!("{ratio:.4}")])
        })
        .collect();

    Some(json!(palette_json))
}
