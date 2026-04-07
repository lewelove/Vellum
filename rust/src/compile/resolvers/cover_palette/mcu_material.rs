use image::DynamicImage;
use mcu_material_color::prelude::*;
use palette::Srgb;

pub fn extract(img: &DynamicImage, args: &str) -> Vec<Srgb> {
    let k = args.split(',')
        .find(|s| s.trim().starts_with("k="))
        .and_then(|s| s.trim().strip_prefix("k="))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(12);

    let pixels: Vec<u32> = img.to_rgba8().pixels().map(|p| {
        let r = p[0] as u32;
        let g = p[1] as u32;
        let b = p[2] as u32;
        let a = p[3] as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }).collect();

    let color_counts = QuantizerCelebi::quantize(&pixels, 128);
    
    let options = ScoreOptions::default().with_desired(k);
    let ranked_colors = Score::score_options(&color_counts, options);
    
    let mut result = Vec::new();
    for color in ranked_colors {
        let r = ((color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((color >> 8) & 0xFF) as f32 / 255.0;
        let b = (color & 0xFF) as f32 / 255.0;
        result.push(Srgb::new(r, g, b));
    }
    
    result
}
