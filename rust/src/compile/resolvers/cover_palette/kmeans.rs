use image::DynamicImage;
use kmeans_colors::get_kmeans_hamerly;
use palette::{FromColor, Lab, Srgb};
use rand::{SeedableRng, rngs::StdRng};
use rand::distr::{Distribution, Uniform};

pub fn extract(img: &DynamicImage, args: &str) -> Vec<(String, f32)> {
    let k = args.split(',')
        .find(|s| s.trim().starts_with("k="))
        .and_then(|s| s.trim().strip_prefix("k="))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(10)
        .clamp(1, 24);

    let noise = args.split(',')
        .find(|s| s.trim().starts_with("noise="))
        .and_then(|s| s.trim().strip_prefix("noise="))
        .and_then(|val| val.parse::<f32>().ok())
        .unwrap_or(0.0);

    let conv = args.split(',')
        .find(|s| s.trim().starts_with("conv="))
        .and_then(|s| s.trim().strip_prefix("conv="))
        .and_then(|val| val.parse::<f32>().ok())
        .unwrap_or(0.000);

    let mut pixels: Vec<Lab> = img.to_rgb8().pixels().map(|p| {
        Lab::from_color(Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ))
    }).collect();

    let total_pixels = pixels.len() as f32;

    if noise > 0.0 {
        let mut rng = StdRng::seed_from_u64(42);
        if let Ok(dist) = Uniform::new_inclusive(-noise, noise) {
            for pixel in &mut pixels {
                let offset = dist.sample(&mut rng);
                pixel.l = (pixel.l + offset).clamp(0.0, 100.0);
            }
        }
    }

    let result = get_kmeans_hamerly(k, 20, conv, false, &pixels, 42);
    let mut counts = vec![0; result.centroids.len()];
    for &idx in &result.indices {
        counts[idx as usize] += 1;
    }

    let mut palette = Vec::new();
    for i in 0..result.centroids.len() {
        let ratio = counts[i] as f32 / total_pixels;
        if ratio > 0.0 {
            let srgb = Srgb::from_color(result.centroids[i]);
            let hex = format!("#{:02X}{:02X}{:02X}", 
                (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8
            );
            palette.push((hex, ratio));
        }
    }

    palette
}
