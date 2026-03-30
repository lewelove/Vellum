pub mod kmeans;
pub mod kmeans_filtered;
pub mod mcu_material;
pub mod mean_shift;

use crate::compile::builder::context::AlbumContext;
use image::imageops::FilterType;
use serde_json::{Value, json};

pub fn resolve(ctx: &AlbumContext, args: &str) -> Option<Value> {
    let cover_path = ctx.cover_path?;
    let img = image::open(ctx.album_root.join(cover_path)).ok()?;

    let sample_dim = args.split(',')
        .find(|s| s.trim().starts_with("dim="))
        .and_then(|s| s.trim().strip_prefix("dim="))
        .and_then(|val| val.parse::<u32>().ok())
        .unwrap_or(512);

    let img_small = img.resize_exact(sample_dim, sample_dim, FilterType::Nearest);

    let algo_type = args.split(',')
        .find(|s| s.trim().starts_with("type="))
        .map(|s| s.trim().strip_prefix("type=").unwrap())
        .unwrap_or("kmeans");

    let mut palette = match algo_type {
        "msc" => mean_shift::extract(&img_small, args),
        "material" => mcu_material::extract(&img_small, args),
        "kmeans_filtered" => kmeans_filtered::extract(&img_small, args),
        "kmeans" | _ => kmeans::extract(&img_small, args),
    };

    palette.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let palette_json: Vec<Value> = palette.into_iter()
        .map(|(hex, ratio)| json!([hex, format!("{ratio:.4}")]))
        .collect();

    Some(json!(palette_json))
}
