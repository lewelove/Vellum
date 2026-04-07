use image::DynamicImage;
use palette::Srgb;
use palette_extract::{get_palette_with_options, PixelEncoding, Quality, MaxColors, PixelFilter};

pub fn extract(img: &DynamicImage, args: &str) -> Vec<Srgb> {
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

    let mut result = Vec::with_capacity(extracted.len());
    for color in extracted {
        result.push(Srgb::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
        ));
    }

    result
}
