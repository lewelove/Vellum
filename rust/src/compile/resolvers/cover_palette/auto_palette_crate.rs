use auto_palette::{Algorithm, ImageData, Palette, Theme};
use image::DynamicImage;
use palette::Srgb;

pub fn extract(img: &DynamicImage, args: &str) -> Vec<Srgb> {
    let k = args.split(',')
        .find(|s| s.trim().starts_with("k="))
        .and_then(|s| s.trim().strip_prefix("k="))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(24);

    let sw = args.split(',')
        .find(|s| s.trim().starts_with("sw="))
        .and_then(|s| s.trim().strip_prefix("sw="))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(128);

    let theme_str = args.split(',')
        .find(|s| s.trim().starts_with("theme="))
        .and_then(|s| s.trim().strip_prefix("theme="))
        .unwrap_or("light");

    let theme_opt = match theme_str.to_lowercase().as_str() {
        "basic" => None,
        "colorful" => Some(Theme::Colorful),
        "vivid" => Some(Theme::Vivid),
        "muted" => Some(Theme::Muted),
        "dark" => Some(Theme::Dark),
        "light" | _ => Some(Theme::Light),
    };

    let width = img.width();
    let height = img.height();
    let pixels = img.to_rgba8().into_raw();

    let image_data = ImageData::new(width, height, &pixels).unwrap();

    let palette: Palette<f32> = Palette::builder()
        .algorithm(Algorithm::DBSCANpp)
        .max_swatches(sw)
        .build(&image_data)
        .unwrap();

    let swatches = match theme_opt {
        Some(theme) => palette.find_swatches_with_theme(k, theme).unwrap_or_default(),
        None => palette.find_swatches(k).unwrap_or_default(),
    };

    let mut result = Vec::with_capacity(swatches.len());
    
    for swatch in swatches {
        let rgb = swatch.color().to_rgb();
        result.push(Srgb::new(
            f32::from(rgb.r) / 255.0,
            f32::from(rgb.g) / 255.0,
            f32::from(rgb.b) / 255.0,
        ));
    }

    result
}
