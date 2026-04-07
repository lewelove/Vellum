use auto_palette::{Algorithm, ImageData, Palette};
use image::DynamicImage;
use palette::{FromColor, Srgb};

pub fn extract(img: &DynamicImage, args: &str) -> Vec<(Srgb, f32)> {
    let k = args.split(',')
        .find(|s| s.trim().starts_with("k="))
        .and_then(|s| s.trim().strip_prefix("k="))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(24);

    let threshold = args.split(',')
        .find(|s| s.trim().starts_with("t="))
        .and_then(|s| s.trim().strip_prefix("t="))
        .and_then(|val| val.parse::<f32>().ok())
        .unwrap_or(0.001);

    let width = img.width();
    let height = img.height();
    let pixels = img.to_rgba8().into_raw();

    let image_data = ImageData::new(width, height, &pixels).unwrap();

    let palette: Palette<f32> = Palette::builder()
        .algorithm(Algorithm::DBSCANpp)
        .max_swatches(k)
        .build(&image_data)
        .unwrap();

    let swatches = palette.swatches();

    let mut result: Vec<(Srgb, f32)> = Vec::with_capacity(swatches.len());
    
    for swatch in swatches {
        let rgb = swatch.color().to_rgb();
        let srgb = Srgb::new(
            f32::from(rgb.r) / 255.0,
            f32::from(rgb.g) / 255.0,
            f32::from(rgb.b) / 255.0,
        );
        result.push((srgb, swatch.ratio()));
    }

    result.retain(|&(_, ratio)| ratio >= threshold);

    let final_total: f32 = result.iter().map(|(_, r)| r).sum();
    if final_total > 0.0 {
        for item in &mut result {
            item.1 /= final_total;
        }
    }

    result
}
