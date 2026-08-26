use egui::Color32;
use palette::{FromColor, Oklab, Srgb};

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub ok100: Color32,
    pub ok200: Color32,
    pub ok300: Color32,
    pub ok400: Color32,
    pub ok500: Color32,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            ok100: Color32::from_rgb(255, 255, 255),
            ok200: Color32::from_rgb(43, 43, 43),
            ok300: Color32::from_rgb(30, 30, 30),
            ok400: Color32::from_rgb(24, 24, 24),
            ok500: Color32::from_rgb(8, 8, 8),
        }
    }
}

#[must_use]
pub fn oklab_to_color32(l: f32, a: f32, b: f32) -> Color32 {
    let oklab = Oklab::new(l, a, b);
    let srgb = Srgb::from_color(oklab);
    let srgb_u8: Srgb<u8> = srgb.into_format();

    Color32::from_rgb(srgb_u8.red, srgb_u8.green, srgb_u8.blue)
}

#[must_use]
pub fn oklch_to_color32(l: f32, c: f32, h_deg: f32) -> Color32 {
    let h_rad = h_deg.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    oklab_to_color32(l, a, b)
}

#[must_use]
pub fn parse_oklch(s: &str) -> Option<Color32> {
    let trimmed = s.trim();
    if !trimmed.starts_with("oklch(") || !trimmed.ends_with(')') {
        return None;
    }

    let inner = &trimmed[6..trimmed.len() - 1];
    let parts: Vec<&str> = inner.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let l = parts[0].parse::<f32>().ok()?;
    let c = parts[1].parse::<f32>().ok()?;
    let h = parts[2].parse::<f32>().ok()?;

    Some(oklch_to_color32(l, c, h))
}
