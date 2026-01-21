pub mod adapter;
pub mod compressor;
pub mod logic;
pub mod scanner;
pub mod writer;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use crate::config::AppConfig;

pub fn run(target_root: PathBuf, config: &AppConfig, _force: bool) -> Result<()> {
    let supported_exts = config.generate.as_ref()
        .map(|g| g.supported_extensions.clone())
        .unwrap_or_else(|| vec![".flac".to_string()]);

    let grouping_keys = config.generate.as_ref()
        .map(|g| g.grouping_keys.clone())
        .unwrap_or_else(|| vec!["ALBUMARTIST".to_string(), "ALBUM".to_string()]);

    // 1. Scan
    println!("Scanning files...");
    let files = scanner::scan_files(&target_root, &supported_exts)?;
    println!("Found {} files.", files.len());

    if files.is_empty() {
        return Ok(());
    }

    // 2. Harvest (Parallel/Sequential based on global pool)
    println!("Harvesting tags...");
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    let tracks: Vec<_> = files.par_iter()
        .map(|path| {
            let res = adapter::read_track_tags(path);
            pb.inc(1);
            res
        })
        .filter_map(|r| r.ok())
        .collect();
    
    pb.finish_with_message("Harvest complete");

    // 3. Group
    println!("Grouping {} tracks...", tracks.len());
    let groups = logic::group_tracks(tracks, &grouping_keys);
    println!("Found {} groups.", groups.len());

    // 4. Process Groups
    println!("Generating metadata...");
    
    let pb_groups = ProgressBar::new(groups.len() as u64);
    let group_list: Vec<_> = groups.into_iter().collect();

    let results: Vec<Result<()>> = group_list.par_iter().map(|(_key, tracks)| {
        // A. Resolve Anchor
        let anchor = match logic::resolve_anchor(tracks, &target_root) {
            Some(a) => a,
            None => return Ok(()),
        };

        // B. Sort
        let mut sorted_tracks = tracks.clone();
        logic::sort_tracks(&mut sorted_tracks);

        // C. Compress
        let (album_pool, track_pools) = compressor::compress(
            &sorted_tracks, 
            config.compress.as_ref().and_then(|c| c.tracks.as_ref())
        );

        // D. Render TOML
        let mut output = String::new();
        output.push_str("[album]\n");
        let alb_lines = writer::render_toml_block(
            &album_pool, 
            config.compress.as_ref().and_then(|c| c.album.as_ref())
        );
        output.push_str(&alb_lines.join("\n"));
        output.push_str("\n\n");

        let trk_layout = config.compress.as_ref().and_then(|c| c.tracks.as_ref());
        for tp in track_pools {
            output.push_str("[[tracks]]\n");
            let trk_lines = writer::render_toml_block(&tp, trk_layout);
            output.push_str(&trk_lines.join("\n"));
            output.push_str("\n\n");
        }

        // E. Write Metadata
        let meta_path = anchor.join("metadata.toml");
        fs::write(&meta_path, output)
            .with_context(|| format!("Failed to write {:?}", meta_path))?;

        // F. Extract Cover
        let cover_candidates = ["cover.jpg", "cover.png", "folder.jpg", "folder.png"];
        let has_cover = cover_candidates.iter().any(|c| anchor.join(c).exists());
        
        if !has_cover && !sorted_tracks.is_empty() {
             let _ = adapter::extract_cover(&sorted_tracks[0].path, &anchor);
        }

        pb_groups.inc(1);
        Ok(())
    }).collect();

    pb_groups.finish_with_message("Generation complete");

    let errors = results.iter().filter(|r| r.is_err()).count();
    if errors > 0 {
        println!("Completed with {} errors.", errors);
    } else {
        println!("Success.");
    }

    Ok(())
}
