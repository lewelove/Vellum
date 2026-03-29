use image::DynamicImage;
use palette::{FromColor, Lab, Srgb};

pub fn extract(img: &DynamicImage, args: &str) -> Vec<(String, f32)> {
    let bw = args.split(',')
        .find(|s| s.trim().starts_with("bw="))
        .and_then(|s| s.trim().strip_prefix("bw="))
        .and_then(|val| val.parse::<f32>().ok())
        .unwrap_or(12.0);
        
    let eps = args.split(',')
        .find(|s| s.trim().starts_with("eps="))
        .and_then(|s| s.trim().strip_prefix("eps="))
        .and_then(|val| val.parse::<f32>().ok())
        .unwrap_or(0.01);
        
    let max_iter = args.split(',')
        .find(|s| s.trim().starts_with("iter="))
        .and_then(|s| s.trim().strip_prefix("iter="))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(20);

    let img_small = img.resize_exact(64, 64, image::imageops::FilterType::Nearest);
    let sample_pixels: Vec<Lab> = img_small.to_rgb8().pixels().map(|p| {
        Lab::from_color(Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ))
    }).collect();
    
    let bw_sq = bw * bw;
    let mut converged = Vec::with_capacity(sample_pixels.len());
    
    for &seed in &sample_pixels {
        let mut current = seed;
        for _ in 0..max_iter {
            let mut sum_l = 0.0;
            let mut sum_a = 0.0;
            let mut sum_b = 0.0;
            let mut total_weight = 0.0;
            
            for &p in &sample_pixels {
                let dist_sq = (current.l - p.l).powi(2) + (current.a - p.a).powi(2) + (current.b - p.b).powi(2);
                if dist_sq < bw_sq {
                    sum_l += p.l;
                    sum_a += p.a;
                    sum_b += p.b;
                    total_weight += 1.0;
                }
            }
            
            if total_weight > 0.0 {
                let next = Lab::new(sum_l / total_weight, sum_a / total_weight, sum_b / total_weight);
                let shift_sq = (next.l - current.l).powi(2) + (next.a - current.a).powi(2) + (next.b - current.b).powi(2);
                current = next;
                if shift_sq < eps {
                    break;
                }
            } else {
                break;
            }
        }
        converged.push(current);
    }
    
    let merge_threshold = 10.0_f32;
    let merge_threshold_sq = merge_threshold * merge_threshold;
    
    let mut centers: Vec<Lab> = Vec::new();
    
    for pos in converged {
        let mut found = false;
        for center in &centers {
            let dist_sq = (pos.l - center.l).powi(2) + (pos.a - center.a).powi(2) + (pos.b - center.b).powi(2);
            if dist_sq < merge_threshold_sq {
                found = true;
                break;
            }
        }
        if !found {
            centers.push(pos);
        }
    }
    
    let full_pixels: Vec<Lab> = img.to_rgb8().pixels().map(|p| {
        Lab::from_color(Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ))
    }).collect();

    let mut counts = vec![0; centers.len()];
    for p in full_pixels.iter() {
        let mut best_idx = 0;
        let mut min_dist_sq = f32::MAX;
        
        for (i, center) in centers.iter().enumerate() {
            let dist_sq = (p.l - center.l).powi(2) + (p.a - center.a).powi(2) + (p.b - center.b).powi(2);
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_idx = i;
            }
        }
        counts[best_idx] += 1;
    }
    
    let full_total = full_pixels.len() as f32;
    let result: Vec<(String, f32)> = centers.into_iter().zip(counts.into_iter()).map(|(lab, count)| {
        let srgb = Srgb::from_color(lab);
        let hex = format!("#{:02X}{:02X}{:02X}", 
            (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
            (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
            (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8
        );
        let ratio = count as f32 / full_total;
        (hex, ratio)
    }).collect();
    
    result
}
