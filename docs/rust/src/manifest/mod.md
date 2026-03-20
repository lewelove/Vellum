File: rust/src/manifest/mod.rs
Role: Manifest Generation Orchestrator

Description:
This file commands the process of discovering unmanaged audio files in the user's library and automatically building the initial `metadata.toml` sidecar files to bring them under the system's management.

Imports:
`pub mod compressor; pub mod engine; pub mod grouper;`
- Exposes internal logic modules.
`use crate::config::AppConfig; use crate::expand_path; use crate::harvest::harvest_file;`
- Connects to settings, path helpers, and physical audio extraction logic.
`use anyhow::{Context, Result};`
- Error handling.
`use indicatif::{ProgressBar, ProgressStyle};`
- Provides visual loading bars for the terminal interface.
`use rayon::prelude::*;`
- Enables multi-threading for maximum performance.
`use std::fs; use std::path::PathBuf;`
- Native file operations.
`use walkdir::WalkDir;`
- A utility to efficiently search nested folders.

Logic:
`pub async fn run(force: bool) -> Result<()>`
- The entry point for the manifest generation routine.
- It first searches the entire library for any folder containing audio files but lacking a `metadata.toml` file (unless forced to scan everything). Once it collects all unmanaged audio files, it multi-threads the `harvest_file` function to quickly rip the physical tags out of them. It passes these harvested files to `grouper::group_tracks` to assemble them into cohesive albums. For each group, it perfectly sorts the tracks, promotes redundant tags (like the album artist) to the top level using `compressor::compress`, and uses `engine::render_toml_block` to write the neatly formatted `metadata.toml` file into the folder. Finally, it triggers a system update so the server immediately recognizes the new album.
