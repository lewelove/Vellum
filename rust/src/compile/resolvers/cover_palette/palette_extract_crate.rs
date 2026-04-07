use image::DynamicImage;
use palette::{FromColor, Oklab, Srgb};
use palette_extract::{get_palette_with_options, PixelEncoding, Quality, MaxColors, PixelFilter};

pub fn extract(img: &DynamicImage, args: &str) -> Vec<(Srgb, f32)> {
    let k = args.split(',')
        .find(|s| s.trim().starts_with("k="))
        .and_then(|s| s.trim().strip_prefix("k="))
        .and_then(|val| val.parse::<u8>().ok())
        .unwrap_or(8)
        .clamp(1, 24);

    let quality_val = args.split(',')
        .find(|s| s.trim().starts_with("q="))
        .and_then(|s| s.trim().strip_prefix("q="))
        .and_then(|val| val.parse::<u8>().ok())
        .unwrap_or(10);

    let img_rgba = img.to_rgba8();
    let pixels = img_rgba.as_raw();

    let extracted = get_palette_with_options(
        pixels, 
        PixelEncoding::Rgba, 
        Quality::new(quality_val), 
        MaxColors::new(k), 
        PixelFilter::None
    );

    let mut palette_colors = Vec::new();
    for color in extracted {
        palette_colors.push(Oklab::from_color(Srgb::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
        )));
    }

    let img_small = img.resize_exact(64, 64, image::imageops::FilterType::Nearest);
    let sample_pixels: Vec<Oklab<f32>> = img_small.to_rgb8().pixels().map(|p| {
        Oklab::from_color(Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ))
    }).collect();

    let mut counts = vec![0.0_f32; palette_colors.len()];
    for p in &sample_pixels {
        let mut best_idx = 0;
        let mut min_dist_sq = f32::MAX;

        for (i, center) in palette_colors.iter().enumerate() {
            let dist_sq = (p.l - center.l).powi(2) + (p.a - center.a).powi(2) + (p.b - center.b).powi(2);
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_idx = i;
            }
        }
        counts[best_idx] += 1.0;
    }

    let total = sample_pixels.len() as f32;
    let result: Vec<(Srgb, f32)> = palette_colors.into_iter().zip(counts.into_iter()).map(|(ok, count)| {
        (Srgb::from_color(ok), count / total)
    }).collect();

    result
}
