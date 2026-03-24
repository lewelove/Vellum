use clap::Parser;
use image::{GenericImageView, imageops::FilterType, DynamicImage};
use minifb::{Window, WindowOptions, Key};
use palette::{FromColor, Lab, Srgb};

#[derive(Parser)]
struct Args {
    path: String,
}

struct ClusterResult {
    centers: Vec<Lab>,
    ratios: Vec<f64>,
    ratio_labels: Vec<usize>,
    bandwidth: f32,
    pre_merge_count: usize,
}

fn run_mean_shift(img: &DynamicImage) -> ClusterResult {
    let discovery_img = img.resize_exact(64, 64, FilterType::Nearest);
    let mut discovery_pixels: Vec<Lab> = Vec::new();
    let mut weights: Vec<f32> = Vec::new();

    for (_, _, p) in discovery_img.pixels() {
        let srgb = Srgb::new(p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0);
        let lab = Lab::from_color(srgb);
        
        // Calculate Chroma: distance from the L-axis (a=0, b=0)
        let chroma = (lab.a.powi(2) + lab.b.powi(2)).sqrt();
        // Saliency boost: vibrant colors get more "mass"
        let weight = 1.0 + (chroma * 0.15); 
        
        discovery_pixels.push(lab);
        weights.push(weight);
    }

    let sample_size = 512;
    let step = (discovery_pixels.len() / sample_size).max(1);
    let seeds: Vec<Lab> = discovery_pixels.iter().step_by(step).take(sample_size).copied().collect();

    let mut distances = Vec::new();
    for i in 0..seeds.len() {
        for j in (i + 1)..seeds.len() {
            let p1 = seeds[i];
            let p2 = seeds[j];
            distances.push(((p1.l - p2.l).powi(2) + (p1.a - p2.a).powi(2) + (p1.b - p2.b).powi(2)).sqrt());
        }
    }
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let bandwidth = if distances.is_empty() { 6.0 } else { 
        distances[(distances.len() as f32 * 0.2) as usize].clamp(2.0, 16.0) 
    };
    
    let bandwidth_sq = bandwidth * bandwidth;
    let convergence_sq = 0.1_f32;
    let max_iter = 15;

    let mut raw_modes = Vec::new();
    for &seed in &seeds {
        let mut current = seed;
        for _ in 0..max_iter {
            let (mut sum_l, mut sum_a, mut sum_b, mut total_weight) = (0.0, 0.0, 0.0, 0.0);
            for (idx, p) in discovery_pixels.iter().enumerate() {
                let dist_sq = (p.l - current.l).powi(2) + (p.a - current.a).powi(2) + (p.b - current.b).powi(2);
                if dist_sq <= bandwidth_sq {
                    let w = weights[idx];
                    sum_l += p.l * w;
                    sum_a += p.a * w;
                    sum_b += p.b * w;
                    total_weight += w;
                }
            }
            if total_weight > 0.0 {
                let next = Lab::new(sum_l / total_weight, sum_a / total_weight, sum_b / total_weight);
                let shift_sq = (next.l - current.l).powi(2) + (next.a - current.a).powi(2) + (next.b - current.b).powi(2);
                current = next;
                if shift_sq < convergence_sq { break; }
            } else { break; }
        }
        raw_modes.push(current);
    }

    let pre_merge_count = raw_modes.len();
    let mut merged_centers: Vec<Lab> = Vec::new();
    // Lower merge threshold to preserve distinct accents
    let merge_threshold_sq = 32.0_f32; 

    for mode in raw_modes {
        let mut is_merged = false;
        for center in &merged_centers {
            let dist_sq = (center.l - mode.l).powi(2) + (center.a - mode.a).powi(2) + (center.b - mode.b).powi(2);
            if dist_sq < merge_threshold_sq {
                is_merged = true;
                break;
            }
        }
        if !is_merged { merged_centers.push(mode); }
    }

    let ratio_img = img.resize_exact(256, 256, FilterType::Nearest);
    let mut physical_counts = vec![0_usize; merged_centers.len()];
    let mut ratio_labels = Vec::new();

    for (_, _, p) in ratio_img.pixels() {
        let pixel_lab = Lab::from_color(Srgb::new(p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0));
        let mut best_dist = f32::MAX;
        let mut best_idx = 0;
        for (idx, center_lab) in merged_centers.iter().enumerate() {
            let dist_sq = (pixel_lab.l - center_lab.l).powi(2) + (pixel_lab.a - center_lab.a).powi(2) + (pixel_lab.b - center_lab.b).powi(2);
            if dist_sq < best_dist {
                best_dist = dist_sq;
                best_idx = idx;
            }
        }
        physical_counts[best_idx] += 1;
        ratio_labels.push(best_idx);
    }

    let total_samples = 256.0 * 256.0;
    let ratios: Vec<f64> = physical_counts.iter().map(|&c| c as f64 / total_samples).collect();

    ClusterResult {
        centers: merged_centers,
        ratios,
        ratio_labels,
        bandwidth,
        pre_merge_count,
    }
}

fn lab_to_u32(lab: Lab) -> u32 {
    let srgb = Srgb::from_color(lab);
    let r = (srgb.red.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (srgb.green.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (srgb.blue.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

fn lab_to_hex(lab: Lab) -> String {
    let srgb = Srgb::from_color(lab);
    format!("#{:02X}{:02X}{:02X}", 
        (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
        (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
        (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8
    )
}

fn main() {
    let args = Args::parse();
    let img = image::open(&args.path).expect("Failed to open image");
    let res = run_mean_shift(&img);

    println!("Bandwidth: {:.2}", res.bandwidth);
    println!("Pre-merge clusters: {}", res.pre_merge_count);
    println!("Post-merge clusters: {}", res.centers.len());

    let mut sorted_indices: Vec<usize> = (0..res.centers.len()).collect();
    sorted_indices.sort_by(|&a, &b| res.ratios[b].partial_cmp(&res.ratios[a]).unwrap());

    println!("\nFinal Palette (Sorted by Representation):");
    for (i, &idx) in sorted_indices.iter().enumerate() {
        println!("  {}: {} | Ratio: {:.4}", i + 1, lab_to_hex(res.centers[idx]), res.ratios[idx]);
    }
    println!("");

    let mut window_buffer = vec![0u32; 768 * 256];

    let original_256 = img.resize_exact(256, 256, FilterType::Nearest);
    for (x, y, p) in original_256.pixels() {
        let color = ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32);
        window_buffer[y as usize * 768 + x as usize] = color;
    }

    for (idx, &cluster_idx) in res.ratio_labels.iter().enumerate() {
        let x = idx % 256;
        let y = idx / 256;
        let color = lab_to_u32(res.centers[cluster_idx]);
        window_buffer[y * 768 + (x + 256)] = color;
    }

    let mut current_pixel = 0;
    for &idx in &sorted_indices {
        let count = (res.ratios[idx] * 256.0 * 256.0).round() as usize;
        let color = lab_to_u32(res.centers[idx]);
        for _ in 0..count {
            if current_pixel >= 256 * 256 { break; }
            let x = current_pixel % 256;
            let y = current_pixel / 256;
            window_buffer[y * 768 + (x + 512)] = color;
            current_pixel += 1;
        }
    }

    let mut window = Window::new(
        "Mean Shift Cluster Debug",
        1536,
        512,
        WindowOptions::default(),
    ).unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&window_buffer, 768, 256).unwrap();
    }
}
