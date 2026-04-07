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

pub fn extract(img: &DynamicImage, args: &str) -> Vec<Srgb> {
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

    let hb = args.split(',').find(|s| s.trim().starts_with("Hb=")).and_then(|s| s.trim().strip_prefix("Hb=")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(35.0);
    let ht = args.split(',').find(|s| s.trim().starts_with("Ht=")).and_then(|s| s.trim().strip_prefix("Ht=")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(100.0);
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

    let result = get_kmeans_hamerly(k, 20, conv, false, &pixels, 42);

    let mut filtered_palette = Vec::new();

    for centroid in result.centroids {
        let srgb = Srgb::from_color(centroid);
        let r = (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8;

        let argb = 0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let hct = Hct::from_int(argb);

        if is_disliked(&hct, hb, ht, cb, ct, tb, tt) {
            if mode == "fix" {
                let fixed = fix_if_disliked(&hct, hb, ht, cb, ct, tb, tt);
                let fixed_argb = fixed.to_int();
                let fr = ((fixed_argb >> 16) & 0xFF) as f32 / 255.0;
                let fg = ((fixed_argb >> 8) & 0xFF) as f32 / 255.0;
                let fb = (fixed_argb & 0xFF) as f32 / 255.0;
                filtered_palette.push(Srgb::new(fr, fg, fb));
            }
        } else {
            let fr = r as f32 / 255.0;
            let fg = g as f32 / 255.0;
            let fb = b as f32 / 255.0;
            filtered_palette.push(Srgb::new(fr, fg, fb));
        }
    }

    filtered_palette
}
