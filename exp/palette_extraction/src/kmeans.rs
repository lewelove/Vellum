use image::{imageops::FilterType, GenericImageView};
use minifb::{Key, Window, WindowOptions};
use palette::{FromColor, Lab, Srgb};

fn parse_arg<T: std::str::FromStr>(args: &str, key: &str, default: T) -> T {
    args.split(',')
        .find(|s| s.trim().starts_with(key))
        .and_then(|s| s.split('=').nth(1))
        .and_then(|v| v.trim().parse::<T>().ok())
        .unwrap_or(default)
}

pub fn run_pure_kmeans(path: &str, arg_str: &str) {
    let k = parse_arg(arg_str, "k", 10);
    let pow = parse_arg(arg_str, "pow", 1.0f32);
    let dim = parse_arg(arg_str, "dim", 512u32);
    let max_iter = parse_arg(arg_str, "iter", 20);
    let eps = parse_arg(arg_str, "eps", 0.00f32);

    let img = image::open(path).expect("Failed to open image");
    let start = std::time::Instant::now();

    let img_small = img.resize_exact(dim, dim, FilterType::Nearest);
    let mut pixels = Vec::with_capacity((dim * dim) as usize);
    let mut weights = Vec::with_capacity((dim * dim) as usize);

    for (_, _, p) in img_small.pixels() {
        let lab = Lab::from_color(Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ));

        let chroma = (lab.a.powi(2) + lab.b.powi(2)).sqrt();
        let gravity = chroma.powf(pow) + 0.01;

        pixels.push(lab);
        weights.push(gravity);
    }

    let mut centroids = Vec::with_capacity(k);
    for i in 0..k {
        centroids.push(pixels[i * (pixels.len() / k)]);
    }

    let mut assignments = vec![0usize; pixels.len()];

    for _ in 0..max_iter {
        for (i, pixel) in pixels.iter().enumerate() {
            let mut min_dist = f32::MAX;
            let mut best_idx = 0;
            for (c_idx, centroid) in centroids.iter().enumerate() {
                let dist = (pixel.l - centroid.l).powi(2)
                    + (pixel.a - centroid.a).powi(2)
                    + (pixel.b - centroid.b).powi(2);
                if dist < min_dist {
                    min_dist = dist;
                    best_idx = c_idx;
                }
            }
            assignments[i] = best_idx;
        }

        let mut new_sums = vec![(0.0f32, 0.0f32, 0.0f32); k];
        let mut weight_sums = vec![0.0f32; k];

        for (i, &c_idx) in assignments.iter().enumerate() {
            let p = &pixels[i];
            let w = weights[i];
            new_sums[c_idx].0 += p.l * w;
            new_sums[c_idx].1 += p.a * w;
            new_sums[c_idx].2 += p.b * w;
            weight_sums[c_idx] += w;
        }

        let mut max_shift = 0.0f32;
        for i in 0..k {
            if weight_sums[i] > 0.0 {
                let next_c = Lab::new(
                    new_sums[i].0 / weight_sums[i],
                    new_sums[i].1 / weight_sums[i],
                    new_sums[i].2 / weight_sums[i],
                );
                let shift = (next_c.l - centroids[i].l).powi(2)
                    + (next_c.a - centroids[i].a).powi(2)
                    + (next_c.b - centroids[i].b).powi(2);
                if shift > max_shift {
                    max_shift = shift;
                }
                centroids[i] = next_c;
            }
        }

        if max_shift < eps {
            break;
        }
    }

    let mut counts = vec![0usize; k];
    for &idx in &assignments {
        counts[idx] += 1;
    }

    let total_px = pixels.len() as f32;
    let mut palette: Vec<(Lab, f32)> = centroids.iter().enumerate()
        .map(|(i, &lab)| (lab, counts[i] as f32 / total_px))
        .collect();

    palette.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Weighted K-Means took: {:?}", start.elapsed());
    println!("\nGravity Palette (k={}, pow={}, dim={}):", k, pow, dim);
    for (i, (lab, ratio)) in palette.iter().enumerate() {
        println!("  {}: {} | Ratio: {:.4}", i + 1, lab_to_hex(*lab), ratio);
    }

    let window_w = (dim * 3) as usize;
    let window_h = dim as usize;
    let mut buffer = vec![0u32; window_w * window_h];

    for y in 0..(dim as usize) {
        for x in 0..(dim as usize) {
            let idx = y * (dim as usize) + x;
            let p = img_small.get_pixel(x as u32, y as u32);
            let orig_col = ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32);
            buffer[y * window_w + x] = orig_col;

            let c_idx = assignments[idx];
            let mapped_col = lab_to_u32(centroids[c_idx]);
            buffer[y * window_w + (x + dim as usize)] = mapped_col;
        }
    }

    let mut curr_ratio_px = 0;
    for (lab, ratio) in &palette {
        let count = (ratio * (dim as f32) * (dim as f32)).round() as usize;
        let color = lab_to_u32(*lab);
        for _ in 0..count {
            if curr_ratio_px >= (dim * dim) as usize { break; }
            let rx = curr_ratio_px % (dim as usize);
            let ry = curr_ratio_px / (dim as usize);
            buffer[ry * window_w + (rx + (dim as usize) * 2)] = color;
            curr_ratio_px += 1;
        }
    }

    let mut window = Window::new(
        "K-Means Manual Weighted: Orig | Weighted | Sorted",
        window_w,
        window_h,
        WindowOptions::default(),
    ).unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, window_w, window_h).unwrap();
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
