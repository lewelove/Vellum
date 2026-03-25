use image::{imageops::FilterType, GenericImageView};
use kmeans_colors::get_kmeans_hamerly;
use minifb::{Key, Window, WindowOptions};
use palette::{FromColor, Lab, Srgb};

const DISCARD_THRESHOLD: f32 = 0.0000; // Roughly < 33 pixels in a 256x256 image

pub fn run_pure_kmeans(path: &str) {
    let img = image::open(path).expect("Failed to open image");
    let start = std::time::Instant::now();

    // 1. 256p Nearest Resize
    let img_small = img.resize_exact(256, 256, FilterType::Nearest);
    let mut pixels = Vec::with_capacity(256 * 256);
    for (_, _, p) in img_small.pixels() {
        pixels.push(Lab::from_color(Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        )));
    }

    // 2. Initial K-Means k=32
    let k = 24;
    let result = get_kmeans_hamerly(k, 20, 0.005, false, &pixels, 42);
    let total_px = (256 * 256) as f32;

    // 3. First Pass: Calculate counts and identify clusters to discard
    let mut counts = vec![0_usize; k];
    for &idx in &result.indices {
        counts[idx as usize] += 1;
    }

    let mut keep_indices = Vec::new();
    let mut discard_indices = Vec::new();

    for i in 0..k {
        let ratio = counts[i] as f32 / total_px;
        if ratio >= DISCARD_THRESHOLD {
            keep_indices.push(i);
        } else {
            discard_indices.push(i);
        }
    }

    // Fallback: If everything is below threshold (unlikely), keep the largest cluster
    if keep_indices.is_empty() {
        let max_idx = counts.iter().enumerate()
            .max_by_key(|&(_, count)| count)
            .map(|(i, _)| i)
            .unwrap_or(0);
        keep_indices.push(max_idx);
        discard_indices.retain(|&i| i != max_idx);
    }

    // 4. Second Pass: Merge discarded clusters into the nearest kept cluster
    let mut final_counts = counts.clone();
    let mut cluster_remap = (0..k).collect::<Vec<usize>>();

    for &d_idx in &discard_indices {
        let d_lab = result.centroids[d_idx];
        
        // Find nearest "keep" centroid in Lab space
        let mut best_target = keep_indices[0];
        let mut min_dist_sq = f32::MAX;

        for &k_idx in &keep_indices {
            let k_lab = result.centroids[k_idx];
            let dist_sq = (d_lab.l - k_lab.l).powi(2) 
                        + (d_lab.a - k_lab.a).powi(2) 
                        + (d_lab.b - k_lab.b).powi(2);
            
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_target = k_idx;
            }
        }

        // Transfer mass
        final_counts[best_target] += counts[d_idx];
        final_counts[d_idx] = 0;
        cluster_remap[d_idx] = best_target;
    }

    println!("K-Means (k=32) + Re-mapped Discards took: {:?}", start.elapsed());
    println!("Significant clusters remaining: {}", keep_indices.len());

    // 5. Generate Final Palette
    let mut palette: Vec<(Lab, f32)> = keep_indices.iter()
        .map(|&i| (result.centroids[i], final_counts[i] as f32 / total_px))
        .collect();
    
    palette.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\nCleaned Palette:");
    for (i, (lab, ratio)) in palette.iter().enumerate() {
        println!("  {}: {} | Ratio: {:.4}", i + 1, lab_to_hex(*lab), ratio);
    }

    // 6. Visualization (1536x512: Orig | Re-mapped | Ratio Sorted)
    let mut buffer = vec![0u32; 1536 * 512];

    let draw_block = |buf: &mut Vec<u32>, px_x: usize, px_y: usize, panel_offset: usize, color: u32| {
        let start_x = panel_offset + (px_x * 2);
        let start_y = px_y * 2;
        for dy in 0..2 {
            for dx in 0..2 {
                buf[(start_y + dy) * 1536 + (start_x + dx)] = color;
            }
        }
    };

    for y in 0..256 {
        for x in 0..256 {
            let idx = y * 256 + x;
            
            // Panel 1: Original
            let p = img_small.get_pixel(x as u32, y as u32);
            let orig_col = ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32);
            draw_block(&mut buffer, x, y, 0, orig_col);

            // Panel 2: Re-mapped (Noise Cleaned)
            let original_cluster = result.indices[idx] as usize;
            let cleaned_cluster = cluster_remap[original_cluster];
            let mapped_col = lab_to_u32(result.centroids[cleaned_cluster]);
            draw_block(&mut buffer, x, y, 512, mapped_col);
        }
    }

    // Panel 3: Pixels Sorted by Ratio
    let mut curr_ratio_px = 0;
    for (lab, ratio) in &palette {
        let count = (ratio * 256.0 * 256.0).round() as usize;
        let color = lab_to_u32(*lab);
        for _ in 0..count {
            if curr_ratio_px >= 256 * 256 { break; }
            draw_block(&mut buffer, curr_ratio_px % 256, curr_ratio_px / 256, 1024, color);
            curr_ratio_px += 1;
        }
    }

    let mut window = Window::new(
        "K-Means Cleaned: Orig | Cleaned | Ratio Sorted",
        1536,
        512,
        WindowOptions::default(),
    ).unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, 1536, 512).unwrap();
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
