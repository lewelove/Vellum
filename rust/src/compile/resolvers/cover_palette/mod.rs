pub mod auto_palette_crate;
pub mod kmeans;
pub mod kmeans_filtered;
pub mod mcu_material;
pub mod mean_shift;
pub mod palette_extract_crate;

use crate::compile::builder::context::AlbumContext;
use image::imageops::FilterType;
use mcu_hct::Hct;
use palette::{FromColor, Oklch};
use serde_json::{Value, json};

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

    let algo_type = args.split(',')
        .find(|s| s.trim().starts_with("type="))
        .map(|s| s.trim().strip_prefix("type=").unwrap())
        .unwrap_or("kmeans");

    let mut palette = match algo_type {
        "auto" => auto_palette_crate::extract(&img_to_process, args),
        "msc" => mean_shift::extract(&img_to_process, args),
        "material" => mcu_material::extract(&img_to_process, args),
        "kmeans_filtered" => kmeans_filtered::extract(&img_to_process, args),
        "mmcq" => palette_extract_crate::extract(&img_to_process, args),
        "kmeans" | _ => kmeans::extract(&img_to_process, args),
    };

    let sort_type = args.split(',')
        .find(|s| s.trim().starts_with("sort="))
        .and_then(|s| s.trim().strip_prefix("sort="))
        .unwrap_or("ratio");

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
