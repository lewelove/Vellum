mod kmeans;
mod kmeansn;
mod kmeansnd;
mod kmeansnh;
mod kmeansnv;
mod mean_shift;

use anyhow::Result;
use image::DynamicImage;
use image::imageops::FilterType;
use libactions::color::{calculate_palette_ratios, sort_palette};
use libactions::payload::{ActionPayload, read_stdin_payload};
use palette::Srgb;
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct ScriptConfig {
    #[serde(rename = "type")]
    algo_type: Option<String>,
    sort: Option<String>,
    #[serde(default)]
    args: String,
    threshold: Option<f32>,
    open_with: Option<String>,
}

fn main() -> Result<()> {
    let payload: ActionPayload<ScriptConfig> = read_stdin_payload()?;
    let script_config = payload.config.action;
    let force = payload.options.split_whitespace().any(|s| s == "--force");

    for item in &payload.albums {
        let album_dir = &item.path;
        let album_lock = &item.lock;

        let cover_path_str = album_lock
            .pointer("/album/covers/main/file/path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("cover.jpg");

        let out_path = album_dir.join("theme.toml");

        if out_path.exists() && !force {
            continue;
        }

        let cover_path = album_dir.join(cover_path_str);

        if !cover_path.exists() {
            continue;
        }

        if let Ok(img) = image::open(&cover_path)
            && let Some(palette) = process_image_to_palette(&img, &script_config)
        {
            let hex_colors: Vec<String> = palette
                .into_iter()
                .map(|(srgb, _)| {
                    let srgb_u8: Srgb<u8> = srgb.into_format();
                    format!(
                        "#{:02X}{:02X}{:02X}",
                        srgb_u8.red, srgb_u8.green, srgb_u8.blue
                    )
                })
                .collect();

            let formatted_bg = if hex_colors.is_empty() {
                String::new()
            } else {
                format!(
                    "\n{}\n",
                    hex_colors
                        .iter()
                        .map(|c| format!("  \"{c}\","))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            };

            let toml_content =
                format!("[album.colors]\n\nbackground = [{formatted_bg}]\n");

            if std::fs::write(&out_path, toml_content).is_ok() {
                let abs_path = out_path.canonicalize().unwrap_or(out_path);
                println!("Created theme.toml at: {}", abs_path.display());

                if let Some(prog) = &script_config.open_with {
                    let cmd_str = format!("{prog} \"{}\"", abs_path.display());
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&cmd_str)
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                }
            }
        }
    }

    Ok(())
}

fn process_image_to_palette(
    img: &DynamicImage,
    cfg: &ScriptConfig,
) -> Option<Vec<(Srgb, f32)>> {
    let algo_type = cfg.algo_type.as_deref().unwrap_or("kmeansnv");
    let sort_type = cfg.sort.as_deref().unwrap_or("gradient");
    let args = &cfg.args;

    let sample_dim = args
        .split(',')
        .find(|s| s.trim().starts_with("dim="))
        .and_then(|s| s.trim().strip_prefix("dim="))
        .and_then(|val| val.parse::<u32>().ok())
        .unwrap_or(512);

    let img_to_process = if sample_dim == 0 {
        img.clone()
    } else {
        img.resize_exact(sample_dim, sample_dim, FilterType::Nearest)
    };

    let candidate_colors = match algo_type {
        "msc" => mean_shift::extract(&img_to_process, args),
        "kmeansn" => kmeansn::extract(&img_to_process, args),
        "kmeansnh" => kmeansnh::extract(&img_to_process, args),
        "kmeansnd" => kmeansnd::extract(&img_to_process, args),
        "kmeansnv" => kmeansnv::extract(&img_to_process, args),
        _ => kmeans::extract(&img_to_process, args),
    };

    if candidate_colors.is_empty() {
        return None;
    }

    let threshold_val = cfg.threshold.unwrap_or(0.001);
    let mut palette =
        calculate_palette_ratios(&img_to_process, candidate_colors, threshold_val);
    sort_palette(&mut palette, sort_type);

    Some(palette)
}
