use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, SubpixelBin, Weight, Wrap};
use palette::{LinSrgb, Srgb};
use swash::scale::{image::Content, Render, ScaleContext, Source, StrikeWith};
use tiny_skia::{Color, Pixmap};

pub fn truncate_with_ellipsis(
    text: &str,
    font_system: &mut cosmic_text::FontSystem,
    metrics: Metrics,
    max_width: f32,
    attrs: &Attrs,
) -> String {
    let mut buffer = Buffer::new(font_system, metrics);
    buffer.set_size(font_system, None, None);
    buffer.set_text(font_system, text, attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(font_system, false);

    let current_width = buffer.layout_runs().next().map(|r| r.line_w).unwrap_or(0.0);
    if current_width <= max_width {
        return text.to_string();
    }

    let mut current_text = text.to_string();
    while !current_text.is_empty() {
        current_text.pop();
        let display = format!("{}…", current_text);
        buffer.set_text(font_system, &display, attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);
        let w = buffer.layout_runs().next().map_or(0.0, |r| r.line_w);
        if w <= max_width {
            return display;
        }
    }
    String::new()
}

pub fn blend_channel(src_lin: f32, bg_lin: f32, mask: u8) -> f32 {
    let alpha = mask as f32 / 255.0;
    src_lin * alpha + bg_lin * (1.0 - alpha)
}

pub fn subpixel_bin_to_float(bin: SubpixelBin) -> f32 {
    match bin {
        SubpixelBin::Zero => 0.0,
        SubpixelBin::One => 0.25,
        SubpixelBin::Two => 0.5,
        SubpixelBin::Three => 0.75,
    }
}

pub fn render_text_blob(
    title: &str,
    artist: &str,
    font_system: &mut cosmic_text::FontSystem,
    swash_context: &mut ScaleContext,
    scale: f32,
) -> Pixmap {
    let width = (190.0 * scale).round() as u32;
    let height = (32.0 * scale).round() as u32;
    let mut pixmap = Pixmap::new(width, height).unwrap();
    
    let bg_srgb = Srgb::new(50u8, 50, 50);
    let bg_lin: LinSrgb<f32> = bg_srgb.into_linear();
    
    let bg_f32: Srgb<f32> = bg_srgb.into_format();
    pixmap.fill(Color::from_rgba(bg_f32.red, bg_f32.green, bg_f32.blue, 1.0).unwrap());

    let mut render_line = |text: &str, baseline_y: f32, size: f32, weight: Weight, color:[f32; 3]| {
        let text_srgb = Srgb::new(color[0], color[1], color[2]);
        let src_lin: LinSrgb<f32> = text_srgb.into_linear();
        
        let attrs = Attrs::new().family(Family::Name("Inter")).weight(weight);
        let metrics = Metrics::new(size * scale, (size + 3.0) * scale);
        
        let truncated = truncate_with_ellipsis(text, font_system, metrics, (190.0 * scale) as f32, &attrs);
        
        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_size(font_system, Some((190.0 * scale) as f32), Some((32.0 * scale) as f32));
        buffer.set_wrap(font_system, Wrap::None);
        buffer.set_text(font_system, &truncated, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0.0, baseline_y * scale), scale);
                let face_id = physical_glyph.cache_key.font_id;
                
                font_system.db().with_face_data(face_id, |data, face_index| {
                    let font_ref = swash::FontRef::from_index(data, face_index as usize).unwrap();
                    let font_size = f32::from_bits(physical_glyph.cache_key.font_size_bits);
                    let x_fract = subpixel_bin_to_float(physical_glyph.cache_key.x_bin);
                    let y_fract = subpixel_bin_to_float(physical_glyph.cache_key.y_bin);

                    let mut scaler = swash_context.builder(font_ref)
                        .size(font_size)
                        .hint(true)
                        .build();

                    let mut renderer = Render::new(&[
                        Source::ColorOutline(0),
                        Source::ColorBitmap(StrikeWith::BestFit),
                        Source::Outline,
                    ]);
                    
                    renderer.format(swash::zeno::Format::Subpixel);
                    renderer.offset(swash::zeno::Vector::new(x_fract, y_fract));

                    if let Some(image) = renderer.render(&mut scaler, physical_glyph.cache_key.glyph_id) {
                        let x_start = physical_glyph.x + image.placement.left;
                        let y_start = physical_glyph.y - image.placement.top;

                        let data_ptr = pixmap.data_mut();
                        let mask_w = image.placement.width as i32;
                        let mask_h = image.placement.height as i32;

                        for row in 0..mask_h {
                            for col in 0..mask_w {
                                let px = x_start + col;
                                let py = y_start + row;
                                if px < 0 || px >= width as i32 || py < 0 || py >= height as i32 { continue; }

                                let pixel_idx = (py as usize * width as usize + px as usize) * 4;
                                
                                match image.content {
                                    Content::SubpixelMask => {
                                        let mask_idx = (row as usize * mask_w as usize + col as usize) * 4;
                                        
                                        let r_lin = blend_channel(src_lin.red, bg_lin.red, image.data[mask_idx]);
                                        let g_lin = blend_channel(src_lin.green, bg_lin.green, image.data[mask_idx + 1]);
                                        let b_lin = blend_channel(src_lin.blue, bg_lin.blue, image.data[mask_idx + 2]);

                                        let out_srgb = Srgb::<u8>::from_linear(LinSrgb::new(r_lin, g_lin, b_lin));

                                        data_ptr[pixel_idx] = out_srgb.red;
                                        data_ptr[pixel_idx + 1] = out_srgb.green;
                                        data_ptr[pixel_idx + 2] = out_srgb.blue;
                                        data_ptr[pixel_idx + 3] = 255;
                                    }
                                    Content::Mask => {
                                        let m = image.data[row as usize * mask_w as usize + col as usize];
                                        
                                        let r_lin = blend_channel(src_lin.red, bg_lin.red, m);
                                        let g_lin = blend_channel(src_lin.green, bg_lin.green, m);
                                        let b_lin = blend_channel(src_lin.blue, bg_lin.blue, m);

                                        let out_srgb = Srgb::<u8>::from_linear(LinSrgb::new(r_lin, g_lin, b_lin));

                                        data_ptr[pixel_idx] = out_srgb.red;
                                        data_ptr[pixel_idx + 1] = out_srgb.green;
                                        data_ptr[pixel_idx + 2] = out_srgb.blue;
                                        data_ptr[pixel_idx + 3] = 255;
                                    }
                                    Content::Color => {
                                        let color_idx = (row as usize * mask_w as usize + col as usize) * 4;
                                        let a = image.data[color_idx + 3] as f32 / 255.0;
                                        
                                        let src_r = image.data[color_idx] as f32 / 255.0;
                                        let src_g = image.data[color_idx + 1] as f32 / 255.0;
                                        let src_b = image.data[color_idx + 2] as f32 / 255.0;
                                        
                                        let emoji_lin: LinSrgb<f32> = Srgb::new(src_r, src_g, src_b).into_linear();
                                        
                                        let r_lin = emoji_lin.red * a + bg_lin.red * (1.0 - a);
                                        let g_lin = emoji_lin.green * a + bg_lin.green * (1.0 - a);
                                        let b_lin = emoji_lin.blue * a + bg_lin.blue * (1.0 - a);
                                        
                                        let out_srgb = Srgb::<u8>::from_linear(LinSrgb::new(r_lin, g_lin, b_lin));
                                        
                                        data_ptr[pixel_idx] = out_srgb.red;
                                        data_ptr[pixel_idx + 1] = out_srgb.green;
                                        data_ptr[pixel_idx + 2] = out_srgb.blue;
                                        data_ptr[pixel_idx + 3] = 255;
                                    }
                                }
                            }
                        }
                    }
                });
            }
        }
    };

    render_line(title, 14.0, 14.0, Weight::NORMAL, [1.0, 1.0, 1.0]);
    render_line(artist, 30.0, 12.0, Weight::NORMAL,[204.0/255.0, 204.0/255.0, 204.0/255.0]);

    pixmap
}
