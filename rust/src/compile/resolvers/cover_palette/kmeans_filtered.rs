use image::DynamicImage;
use kmeans_colors::get_kmeans_hamerly;
use mcu_hct::Hct;
use palette::{FromColor, Lab, Srgb};

fn is_disliked(hct: &Hct, hb: f64, ht: f64, cb: f64, ct: f64, tb: f64, tt: f64) -> bool {
    let hue = hct.hue();
    let chroma = hct.chroma();
    let tone = hct.tone();

    let hue_passes = hue >= hb && hue <= ht;
    let chroma_passes = chroma >= cb && chroma <= ct;
    let tone_passes = tone >= tb && tone <= tt;

    hue_passes && chroma_passes && tone_passes
}

fn fix_if_disliked(hct: &Hct, hb: f64, ht: f64, cb: f64, ct: f64, tb: f64, tt: f64) -> Hct {
    if is_disliked(hct, hb, ht, cb, ct, tb, tt) {
        Hct::from(hct.hue(), hct.chroma(), 70.0)
    } else {
        *hct
    }
}

pub fn extract(img: &DynamicImage, args: &str) -> Vec<(String, f32)> {
    let k = args.split(',')
        .find(|s| s.trim().starts_with("k="))
        .and_then(|s| s.trim().strip_prefix("k="))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(10)
        .clamp(1, 24);

    let conv = args.split(',')
        .find(|s| s.trim().starts_with("conv="))
        .and_then(|s| s.trim().strip_prefix("conv="))
        .and_then(|val| val.parse::<f32>().ok())
        .unwrap_or(0.005);

    let mode = args.split(',')
        .find(|s| s.trim().starts_with("mode="))
        .and_then(|s| s.trim().strip_prefix("mode="))
        .unwrap_or("cut");

    // HCT Filter Bounds - Parsed as f64 to match Hct internal types
    let hb = args.split(',').find(|s| s.trim().starts_with("Hb=")).and_then(|s| s.trim().strip_prefix("Hb=")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(35.0);
    let ht = args.split(',').find(|s| s.trim().starts_with("Ht=")).and_then(|s| s.trim().strip_prefix("Ht=")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(111.0);
    let cb = args.split(',').find(|s| s.trim().starts_with("Cb=")).and_then(|s| s.trim().strip_prefix("Cb=")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(12.0);
    let ct = args.split(',').find(|s| s.trim().starts_with("Ct=")).and_then(|s| s.trim().strip_prefix("Ct=")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(60.0);
    let tb = args.split(',').find(|s| s.trim().starts_with("Tb=")).and_then(|s| s.trim().strip_prefix("Tb=")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    let tt = args.split(',').find(|s| s.trim().starts_with("Tt=")).and_then(|s| s.trim().strip_prefix("Tt=")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(50.0);

    let pixels: Vec<Lab> = img.to_rgb8().pixels().map(|p| {
        Lab::from_color(Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ))
    }).collect();

    let total_pixels = pixels.len() as f32;

    let result = get_kmeans_hamerly(k, 20, conv, false, &pixels, 42);

    let mut counts = vec![0; result.centroids.len()];
    for &idx in &result.indices {
        counts[idx as usize] += 1;
    }

    let mut all_colors = Vec::new();
    for i in 0..result.centroids.len() {
        let ratio = counts[i] as f32 / total_pixels;
        if ratio > 0.0 {
            let srgb = Srgb::from_color(result.centroids[i]);
            let r = (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8;

            let argb = 0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            let hct = Hct::from_int(argb);

            all_colors.push((r, g, b, hct, ratio));
        }
    }

    all_colors.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap_or(std::cmp::Ordering::Equal));

    let mut filtered_palette = Vec::new();
    let mut total_filtered_ratio = 0.0;

    for (r, g, b, hct, ratio) in &all_colors {
        if is_disliked(hct, hb, ht, cb, ct, tb, tt) {
            if mode == "fix" {
                let fixed = fix_if_disliked(hct, hb, ht, cb, ct, tb, tt);
                let fixed_argb = fixed.to_int();
                let hex = format!("#{:06X}", fixed_argb & 0xFFFFFF);
                filtered_palette.push((hex, *ratio));
                total_filtered_ratio += ratio;
            }
        } else {
            let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
            filtered_palette.push((hex, *ratio));
            total_filtered_ratio += ratio;
        }
    }

    if filtered_palette.is_empty() && !all_colors.is_empty() {
        let (r, g, b, _, ratio) = &all_colors[0];
        let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
        filtered_palette.push((hex, *ratio));
        total_filtered_ratio += ratio;
    }

    if total_filtered_ratio > 0.0 {
        for item in &mut filtered_palette {
            item.1 /= total_filtered_ratio;
        }
    }

    filtered_palette
}
