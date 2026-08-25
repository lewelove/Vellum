use image::DynamicImage;
use palette::{FromColor, Oklab, Srgb};

struct MeanShiftParams {
    bw: f32,
    eps: f32,
    max_iter: usize,
    merge_threshold: f32,
    chroma_gravity: f32,
    k: usize,
}

fn parse_arg<T: std::str::FromStr>(args: &str, prefix: &str, default: T) -> T {
    args.split(',')
        .find(|s| s.trim().starts_with(prefix))
        .and_then(|s| s.trim().strip_prefix(prefix))
        .and_then(|val| val.parse::<T>().ok())
        .unwrap_or(default)
}

fn parse_params(args: &str) -> MeanShiftParams {
    MeanShiftParams {
        bw: parse_arg(args, "bw=", 0.12),
        eps: parse_arg(args, "eps=", 0.0001),
        max_iter: parse_arg(args, "iter=", 20),
        merge_threshold: parse_arg(args, "mt=", 0.10),
        chroma_gravity: parse_arg(args, "cg=", 0.0),
        k: parse_arg(args, "k=", 0),
    }
}

fn sample_pixels(img: &DynamicImage) -> Vec<Oklab> {
    let img_small = img.resize_exact(64, 64, image::imageops::FilterType::Nearest);
    img_small
        .to_rgb8()
        .pixels()
        .map(|p| {
            Oklab::from_color(Srgb::new(
                f32::from(p[0]) / 255.0,
                f32::from(p[1]) / 255.0,
                f32::from(p[2]) / 255.0,
            ))
        })
        .collect()
}

fn shift_point(
    mut current: Oklab,
    samples: &[Oklab],
    params: &MeanShiftParams,
    bw_sq: f32,
) -> Oklab {
    for _ in 0..params.max_iter {
        let mut sum_l = 0.0;
        let mut sum_a = 0.0;
        let mut sum_b = 0.0;
        let mut total_weight = 0.0;

        for &p in samples {
            let dist_sq = (current.b - p.b).mul_add(
                current.b - p.b,
                (current.a - p.a).mul_add(current.a - p.a, (current.l - p.l).powi(2)),
            );
            if dist_sq < bw_sq {
                let chroma = p.a.hypot(p.b);
                let weight = params.chroma_gravity.mul_add(chroma, 1.0);

                sum_l = p.l.mul_add(weight, sum_l);
                sum_a = p.a.mul_add(weight, sum_a);
                sum_b = p.b.mul_add(weight, sum_b);
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            let next = Oklab::new(
                sum_l / total_weight,
                sum_a / total_weight,
                sum_b / total_weight,
            );
            let shift_sq = (next.b - current.b).mul_add(
                next.b - current.b,
                (next.a - current.a)
                    .mul_add(next.a - current.a, (next.l - current.l).powi(2)),
            );
            current = next;
            if shift_sq < params.eps {
                break;
            }
        } else {
            break;
        }
    }
    current
}

fn merge_centers(converged: Vec<Oklab>, threshold: f32, k: usize) -> Vec<Oklab> {
    let merge_threshold_sq = threshold * threshold;
    let mut centers: Vec<Oklab> = Vec::new();

    for pos in converged {
        let mut found = false;
        for center in &centers {
            let dist_sq = (pos.b - center.b).mul_add(
                pos.b - center.b,
                (pos.a - center.a).mul_add(pos.a - center.a, (pos.l - center.l).powi(2)),
            );
            if dist_sq < merge_threshold_sq {
                found = true;
                break;
            }
        }
        if !found {
            centers.push(pos);
        }
    }

    centers.sort_by(|a, b| {
        let chroma_a = a.a.hypot(a.b);
        let chroma_b = b.a.hypot(b.b);
        chroma_b
            .partial_cmp(&chroma_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if k > 0 && centers.len() > k {
        centers.truncate(k);
    }

    centers
}

pub fn extract(img: &DynamicImage, args: &str) -> Vec<Srgb> {
    let params = parse_params(args);
    let samples = sample_pixels(img);
    let bw_sq = params.bw * params.bw;

    let converged: Vec<Oklab> = samples
        .iter()
        .map(|&seed| shift_point(seed, &samples, &params, bw_sq))
        .collect();

    let centers = merge_centers(converged, params.merge_threshold, params.k);
    centers.into_iter().map(Srgb::from_color).collect()
}
