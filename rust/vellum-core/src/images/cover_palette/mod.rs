pub mod kmeans;
pub mod kmeansn;
pub mod kmeansnh;
pub mod kmeansnd;
pub mod kmeansnv;
pub mod mean_shift;

use image::imageops::FilterType;
use image::DynamicImage;
use palette::{FromColor, Oklab, Oklch, Srgb};
use serde_json::{Value, json};

fn parse_hex(hex: &str) -> Option<Srgb> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
}

pub fn process_image_to_palette(
    img: &DynamicImage,
    cfg: &Value,
    mut candidate_colors: Vec<Srgb>,
    manually_provided: bool,
) -> Option<Vec<(Srgb, f32)>> {
    let algo_type = cfg.get("type").and_then(Value::as_str).unwrap_or("kmeansnv");
    let sort_type = cfg.get("sort").and_then(Value::as_str).unwrap_or("gradient");
    let args = cfg.get("args").and_then(Value::as_str).unwrap_or("");

    let sample_dim = args.split(',')
        .find(|s| s.trim().starts_with("dim="))
        .and_then(|s| s.trim().strip_prefix("dim="))
        .and_then(|val| val.parse::<u32>().ok())
        .unwrap_or(512);

    let img_to_process = if sample_dim == 0 {
        img.clone()
    } else {
        img.resize_exact(sample_dim, sample_dim, FilterType::Nearest)
    };

    if candidate_colors.is_empty() {
        candidate_colors = match algo_type {
            "msc" => mean_shift::extract(&img_to_process, args),
            "kmeansn" => kmeansn::extract(&img_to_process, args),
            "kmeansnh" => kmeansnh::extract(&img_to_process, args),
            "kmeansnd" => kmeansnd::extract(&img_to_process, args),
            "kmeansnv" => kmeansnv::extract(&img_to_process, args),
            "kmeans" | _ => kmeans::extract(&img_to_process, args),
        };
    }

    if candidate_colors.is_empty() {
        return None;
    }

    let oklab_centers: Vec<Oklab> = candidate_colors.iter().map(|&c| Oklab::from_color(c)).collect();
    let mut counts = vec![0usize; oklab_centers.len()];

    if !manually_provided {
        for p in img_to_process.to_rgb8().pixels() {
            let pixel_oklab = Oklab::from_color(Srgb::new(
                p[0] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[2] as f32 / 255.0,
            ));
            let mut best_idx = 0;
            let mut min_dist_sq = f32::MAX;
            for (i, center) in oklab_centers.iter().enumerate() {
                let dist_sq = (pixel_oklab.l - center.l).powi(2)
                            + (pixel_oklab.a - center.a).powi(2)
                            + (pixel_oklab.b - center.b).powi(2);
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    best_idx = i;
                }
            }
            counts[best_idx] += 1;
        }
    }

    let total_pixels = counts.iter().sum::<usize>() as f32;
    let mut palette: Vec<(Srgb, f32)> = candidate_colors.into_iter().zip(counts.into_iter()).filter_map(|(color, count)| {
        let ratio = if total_pixels > 0.0 {
            count as f32 / total_pixels
        } else {
            0.0
        };
        
        if manually_provided || ratio > 0.0 {
            Some((color, ratio))
        } else {
            None
        }
    }).collect();

    let threshold_val = cfg.get("threshold")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.001);

    if !manually_provided {
        palette.retain(|&(_, ratio)| ratio >= threshold_val);
    }

    let final_total: f32 = palette.iter().map(|(_, r)| r).sum();
    if final_total > 0.0 {
        for item in &mut palette {
            item.1 /= final_total;
        }
    } else if manually_provided && !palette.is_empty() {
        let even = 1.0 / palette.len() as f32;
        for item in &mut palette {
            item.1 = even;
        }
    }

    let sort_override = if manually_provided {
        "original"
    } else {
        sort_type
    };

    match sort_override {
        "L" => palette.sort_by(|a, b| {
            let l_a = Oklch::from_color(a.0).l;
            let l_b = Oklch::from_color(b.0).l;
            l_b.partial_cmp(&l_a).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "C" => palette.sort_by(|a, b| {
            let c_a = Oklch::from_color(a.0).chroma;
            let c_b = Oklch::from_color(b.0).chroma;
            c_b.partial_cmp(&c_a).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "H" => palette.sort_by(|a, b| {
            let h_a = Oklch::from_color(a.0).hue.into_raw_degrees();
            let h_b = Oklch::from_color(b.0).hue.into_raw_degrees();
            h_a.partial_cmp(&h_b).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "LC" => palette.sort_by(|a, b| {
            let oklch_a = Oklch::from_color(a.0);
            let oklch_b = Oklch::from_color(b.0);
            let val_a = oklch_a.l * oklch_a.chroma;
            let val_b = oklch_b.l * oklch_b.chroma;
            val_b.partial_cmp(&val_a).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "gradient" => {
            if !palette.is_empty() {
                let mut pool: Vec<(Oklab, Srgb, f32)> = palette.into_iter()
                    .map(|(srgb, ratio)| (Oklab::from_color(srgb), srgb, ratio))
                    .collect();

                let mut sorted = Vec::with_capacity(pool.len());
                
                let start_idx = pool.iter().enumerate()
                    .max_by(|(_, (ok_a, _, _)), (_, (ok_b, _, _))| {
                        ok_a.l.partial_cmp(&ok_b.l).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                let first = pool.remove(start_idx);
                let mut current_ok = first.0;
                sorted.push((first.1, first.2));

                let end_node_idx = if pool.is_empty() {
                    None
                } else {
                    pool.iter().enumerate()
                        .min_by(|(_, (ok_a, _, _)), (_, (ok_b, _, _))| {
                            ok_a.l.partial_cmp(&ok_b.l).unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                };

                let end_node = end_node_idx.map(|idx| pool.remove(idx));

                while !pool.is_empty() {
                    let next_idx = pool.iter().enumerate()
                        .min_by(|(_, (ok_a, _, _)), (_, (ok_b, _, _))| {
                            let dist_a = (ok_a.l - current_ok.l).powi(2) + (ok_a.a - current_ok.a).powi(2) + (ok_a.b - current_ok.b).powi(2);
                            let dist_b = (ok_b.l - current_ok.l).powi(2) + (ok_b.a - current_ok.a).powi(2) + (ok_b.b - current_ok.b).powi(2);
                            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap();
                    
                    let next = pool.remove(next_idx);
                    current_ok = next.0;
                    sorted.push((next.1, next.2));
                }

                if let Some(node) = end_node {
                    sorted.push((node.1, node.2));
                }

                palette = sorted;
            }
        },
        "original" => {},
        "ratio" | _ => palette.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)),
    }

    Some(palette)
}

pub fn resolve_core(img: &DynamicImage, cfg: Option<&Value>, cover_palette_raw: Option<&Value>) -> Option<Value> {
    let default_cfg = json!({});
    let cfg_val = cfg.unwrap_or(&default_cfg);

    let mut candidate_colors = Vec::new();
    let mut manually_provided = false;
    let mut should_extract = false;

    match cover_palette_raw {
        Some(Value::Bool(b)) => {
            should_extract = *b;
        }
        Some(Value::String(s)) => {
            let s_lower = s.trim().to_lowercase();
            if s_lower == "true" {
                should_extract = true;
            }
        }
        Some(Value::Array(arr)) => {
            for v in arr {
                if let Some(s) = v.as_str() {
                    if let Some(srgb) = parse_hex(s) {
                        candidate_colors.push(srgb);
                    }
                }
            }
            if !candidate_colors.is_empty() {
                manually_provided = true;
            }
        }
        _ => {}
    }

    if !should_extract && !manually_provided {
        return None;
    }

    let palette = process_image_to_palette(img, cfg_val, candidate_colors, manually_provided)?;

    let palette_json: Vec<Value> = palette.into_iter()
        .map(|(srgb, ratio)| {
            let r_u8 = (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8;
            let g_u8 = (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8;
            let b_u8 = (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8;

            let hex = format!("#{:02X}{:02X}{:02X}", r_u8, g_u8, b_u8);

            let oklch = Oklch::from_color(srgb);
            let l_pct = oklch.l * 100.0;
            let c = oklch.chroma;
            let h = oklch.hue.into_raw_degrees();
            let oklch_str = format!("oklch({:.2}% {:.3} {:.2})", l_pct, c, h);

            json!([hex, oklch_str, format!("{ratio:.4}")])
        })
        .collect();

    Some(json!(palette_json))
}
