pub mod compressor;
pub mod engine;
pub mod grouper;

use crate::config::AppConfig;
use crate::expand_path;
use crate::harvest::harvest_file;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub async fn run(force: bool) -> Result<()> {
    let (config, _raw_config, _) = AppConfig::load().context("Failed to load config")?;
    let lib_root = expand_path(&config.storage.library_root).canonicalize()?;

    let manifest_cfg = config.manifest.context("Missing [manifest] configuration")?;
    let supported_exts: Vec<String> = manifest_cfg
        .supported_extensions
        .iter()
        .map(|e| e.to_lowercase())
        .collect();
    let grouping_keys = vec!["ALBUMARTIST".to_string(), "ALBUM".to_string()];

    let manifest_layout = manifest_cfg.keys;

    let mut dirs_to_harvest = Vec::new();
    let mut it = WalkDir::new(&lib_root).into_iter();

    while let Some(Ok(entry)) = it.next() {
        if entry.file_type().is_dir() {
            let path = entry.path();
            if !force && path.join("metadata.toml").exists() {
                it.skip_current_dir();
                continue;
            }

            let has_audio = fs::read_dir(path)
                .map(|mut d| {
                    d.any(|e| {
                        if let Ok(f) = e {
                            if f.file_type().map_or(false, |ft| ft.is_file()) {
                                if let Some(ext) = f.path().extension().and_then(|e| e.to_str()) {
                                    let ext_lower = format!(".{}", ext.to_lowercase());
                                    return supported_exts.contains(&ext_lower);
                                }
                            }
                        }
                        false
                    })
                })
                .unwrap_or(false);

            if has_audio {
                dirs_to_harvest.push(path.to_path_buf());
            }
        }
    }

    if dirs_to_harvest.is_empty() {
        log::info!("No new audio directories found.");
        return Ok(());
    }

    let mut audio_files = Vec::new();
    for dir in dirs_to_harvest {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = format!(".{}", ext.to_lowercase());
                        if supported_exts.contains(&ext_lower) {
                            audio_files.push(path);
                        }
                    }
                }
            }
        }
    }

    log::info!("Harvesting {} new audio files...", audio_files.len());

    let harvested: Vec<(PathBuf, serde_json::Map<String, serde_json::Value>)> = audio_files
        .into_par_iter()
        .filter_map(|path| match harvest_file(&path) {
            Ok(data) => {
                let mut map = serde_json::Map::new();
                for (k, v) in data.tags {
                    map.insert(k, serde_json::Value::String(v));
                }
                Some((path, map))
            }
            Err(e) => {
                log::warn!("Failed to harvest {}: {}", path.display(), e);
                None
            }
        })
        .collect();

    if harvested.is_empty() {
        return Ok(());
    }

    let buckets = grouper::group_tracks(harvested, &grouping_keys);

    let pb = ProgressBar::new(buckets.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green}[{elapsed_precise}][{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    for (_group_id, mut tracks) in buckets {
        pb.inc(1);
        let (anchor_opt, is_valid) = grouper::resolve_anchor(&tracks, &lib_root, &supported_exts);
        if !is_valid {
            continue;
        }

        if let Some(anchor) = anchor_opt {
            grouper::sort_album_tracks(&mut tracks);
            let clean_tracks: Vec<_> = tracks
                .into_iter()
                .map(|(_, mut t)| {
                    t.remove("track_path_absolute");
                    t
                })
                .collect();

            let (album_pool, track_pools) =
                compressor::compress(clean_tracks, manifest_layout.as_ref());

            let mut toml_content = String::new();
            toml_content.push_str("[album]\n");
            let album_lines = engine::render_toml_block(&album_pool, manifest_layout.as_ref(), "album");
            toml_content.push_str(&album_lines.join("\n"));
            toml_content.push_str("\n\n");

            for tp in track_pools {
                toml_content.push_str("[[tracks]]\n");
                let track_lines = engine::render_toml_block(&tp, manifest_layout.as_ref(), "track");
                toml_content.push_str(&track_lines.join("\n"));
                toml_content.push_str("\n\n");
            }

            let meta_path = anchor.join("metadata.toml");
            if let Err(e) = fs::write(&meta_path, toml_content) {
                log::error!("Failed to write {}: {}", meta_path.display(), e);
            }
        }
    }

    pb.finish_with_message("Generation complete");

    log::info!("Manifest generation complete. Triggering library update...");
    crate::update::run(None, false, None).await?;

    Ok(())
}
