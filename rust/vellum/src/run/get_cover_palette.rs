use anyhow::{Context, Result};
use vellum_core::config::AppConfig;
use crate::compile::builder::assets::COVER_CANDIDATES;
use crate::compile::resolvers::cover_palette;
use std::path::Path;
use serde_json::json;

pub async fn run(config: &AppConfig, target_album: &Path) -> Result<()> {
    let mut cover_path = None;
    for c in COVER_CANDIDATES {
        let p = target_album.join(c);
        if p.exists() {
            cover_path = Some(p);
            break;
        }
    }

    let Some(cp) = cover_path else {
        anyhow::bail!("No cover image found in {}", target_album.display());
    };

    let img = image::open(&cp).context("Failed to open cover image")?;

    let default_cfg = json!({
        "type": "kmeansnv",
        "sort": "gradient",
        "args": "k=24,n=6,d=0.2",
        "threshold": 0.02
    });

    let cfg = config
        .compiler
        .as_ref()
        .and_then(|c| c.cover_palette.clone())
        .map(|p| serde_json::to_value(p).unwrap_or(default_cfg.clone()))
        .unwrap_or(default_cfg);

    let palette = cover_palette::process_image_to_palette(&img, &cfg, Vec::new(), false)
        .context("Failed to extract palette")?;

    println!("[album]\n");
    println!("COVER_PALETTE = [");
    for (srgb, _) in palette {
        let r = (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
        println!("  \"#{:02X}{:02X}{:02X}\",", r, g, b);
    }
    println!("]");

    Ok(())
}
