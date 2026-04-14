use crate::egui::theme;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight, Wrap};
use eframe::egui;
use palette::{LinSrgb, Srgb};
use swash::scale::{image::Content, Render, ScaleContext, Source, StrikeWith};
use tiny_skia::Pixmap;

pub fn truncate_with_ellipsis(
    text: &str,
    font_system: &mut FontSystem,
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

fn subpixel_bin_to_float(bin: cosmic_text::SubpixelBin) -> f32 {
    match bin {
        cosmic_text::SubpixelBin::Zero => 0.0,
        cosmic_text::SubpixelBin::One => 0.25,
        cosmic_text::SubpixelBin::Two => 0.5,
        cosmic_text::SubpixelBin::Three => 0.75,
    }
}

pub fn render_text_blob(
    title: &str,
    artist: &str,
    font_system: &mut FontSystem,
    swash_context: &mut ScaleContext,
    scale: f32,
    width_pts: f32,
    height_pts: f32,
    text_gamma: f32,
    text_magic: f32,
) -> Pixmap {
    let width = (width_pts * scale).round() as u32;
    let height = (height_pts * scale).round() as u32;
    let mut pixmap = Pixmap::new(width, height).unwrap();

    let coverage_gamma = text_gamma;
    let softening = text_magic;

    let mut render_line =
        |text: &str, baseline_y: f32, size: f32, weight: Weight, color: egui::Color32| {
            let text_srgb = Srgb::new(color.r(), color.g(), color.b());
            let src_lin: LinSrgb<f32> = text_srgb.into_linear();

            let attrs = Attrs::new().family(Family::Name("Inter")).weight(weight);
            let metrics = Metrics::new(size * scale, (size + 3.0) * scale);

            let truncated =
                truncate_with_ellipsis(text, font_system, metrics, width_pts * scale, &attrs);

            let mut buffer = Buffer::new(font_system, metrics);
            buffer.set_size(font_system, Some(width_pts * scale), Some(height_pts * scale));
            buffer.set_wrap(font_system, Wrap::None);
            buffer.set_text(font_system, &truncated, &attrs, Shaping::Advanced, None);
            buffer.shape_until_scroll(font_system, false);

            for run in buffer.layout_runs() {
                for glyph in run.glyphs.iter() {
                    let physical_glyph = glyph.physical((0.0, baseline_y * scale), scale);
                    let face_id = physical_glyph.cache_key.font_id;

                    font_system.db().with_face_data(face_id, |data, face_index| {
                        let font_ref =
                            swash::FontRef::from_index(data, face_index as usize).unwrap();
                        let font_size = f32::from_bits(physical_glyph.cache_key.font_size_bits);
                        let x_fract = subpixel_bin_to_float(physical_glyph.cache_key.x_bin);
                        let y_fract = subpixel_bin_to_float(physical_glyph.cache_key.y_bin);

                        let mut scaler = swash_context
                            .builder(font_ref)
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

                        if let Some(image) =
                            renderer.render(&mut scaler, physical_glyph.cache_key.glyph_id)
                        {
                            let x_start = physical_glyph.x + image.placement.left;
                            let y_start = physical_glyph.y - image.placement.top;

                            let data_ptr = pixmap.data_mut();
                            let mask_w = image.placement.width as i32;
                            let mask_h = image.placement.height as i32;

                            for row in 0..mask_h {
                                for col in 0..mask_w {
                                    let px = x_start + col;
                                    let py = y_start + row;
                                    if px < 0 || px >= width as i32 || py < 0 || py >= height as i32
                                    {
                                        continue;
                                    }

                                    let pixel_idx = (py as usize * width as usize + px as usize) * 4;

                                    match image.content {
                                        Content::SubpixelMask => {
                                            let mask_idx =
                                                (row as usize * mask_w as usize + col as usize) * 4;

                                            let r_raw = f32::from(image.data[mask_idx + 2]) / 255.0;
                                            let g_raw = f32::from(image.data[mask_idx + 1]) / 255.0;
                                            let b_raw = f32::from(image.data[mask_idx]) / 255.0;

                                            let avg = (r_raw + g_raw + b_raw) / 3.0;

                                            let r_s = r_raw * (1.0 - softening) + avg * softening;
                                            let g_s = g_raw * (1.0 - softening) + avg * softening;
                                            let b_s = b_raw * (1.0 - softening) + avg * softening;

                                            let m_r = r_s.powf(coverage_gamma);
                                            let m_g = g_s.powf(coverage_gamma);
                                            let m_b = b_s.powf(coverage_gamma);

                                            let alpha_lin = m_r.max(m_g).max(m_b);

                                            let out_r_lin = src_lin.red * m_r;
                                            let out_g_lin = src_lin.green * m_g;
                                            let out_b_lin = src_lin.blue * m_b;

                                            let out_srgb = Srgb::<u8>::from_linear(LinSrgb::new(
                                                out_r_lin, out_g_lin, out_b_lin,
                                            ));

                                            data_ptr[pixel_idx] = out_srgb.red;
                                            data_ptr[pixel_idx + 1] = out_srgb.green;
                                            data_ptr[pixel_idx + 2] = out_srgb.blue;
                                            data_ptr[pixel_idx + 3] = (alpha_lin * 255.0).round() as u8;
                                        }
                                        Content::Mask => {
                                            let m_raw = f32::from(
                                                image.data
                                                    [row as usize * mask_w as usize + col as usize],
                                            ) / 255.0;

                                            let m = m_raw.powf(coverage_gamma);
                                            let alpha_lin = m;

                                            let out_r_lin = src_lin.red * m;
                                            let out_g_lin = src_lin.green * m;
                                            let out_b_lin = src_lin.blue * m;

                                            let out_srgb = Srgb::<u8>::from_linear(LinSrgb::new(
                                                out_r_lin, out_g_lin, out_b_lin,
                                            ));

                                            data_ptr[pixel_idx] = out_srgb.red;
                                            data_ptr[pixel_idx + 1] = out_srgb.green;
                                            data_ptr[pixel_idx + 2] = out_srgb.blue;
                                            data_ptr[pixel_idx + 3] = (alpha_lin * 255.0).round() as u8;
                                        }
                                        Content::Color => {
                                            let color_idx =
                                                (row as usize * mask_w as usize + col as usize) * 4;
                                            let a_raw = f32::from(image.data[color_idx + 3]) / 255.0;

                                            let src_r = f32::from(image.data[color_idx]) / 255.0;
                                            let src_g = f32::from(image.data[color_idx + 1]) / 255.0;
                                            let src_b = f32::from(image.data[color_idx + 2]) / 255.0;

                                            let emoji_lin: LinSrgb<f32> =
                                                Srgb::new(src_r, src_g, src_b).into_linear();

                                            let out_r_lin = emoji_lin.red * a_raw;
                                            let out_g_lin = emoji_lin.green * a_raw;
                                            let out_b_lin = emoji_lin.blue * a_raw;

                                            let out_srgb = Srgb::<u8>::from_linear(LinSrgb::new(
                                                out_r_lin, out_g_lin, out_b_lin,
                                            ));

                                            data_ptr[pixel_idx] = out_srgb.red;
                                            data_ptr[pixel_idx + 1] = out_srgb.green;
                                            data_ptr[pixel_idx + 2] = out_srgb.blue;
                                            data_ptr[pixel_idx + 3] = (a_raw * 255.0).round() as u8;
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            }
        };

    render_line(title, 14.0, 14.0, Weight::NORMAL, theme::TEXT_MAIN);
    render_line(artist, 30.0, 12.0, Weight::NORMAL, theme::TEXT_MUTED);

    pixmap
}
