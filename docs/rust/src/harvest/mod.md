File: rust/src/harvest/mod.rs
Role: Raw Audio Data Harvester

Description:
This module physically inspects audio files to extract bit-perfect technical properties (duration, format, sample rate) and reads any pre-existing native metadata tags embedded in the audio file headers. 

Imports:
`use anyhow::{Context, Result};`
`use lofty::config::ParseOptions;`
`use lofty::file::AudioFile;`
`use lofty::prelude::*;`
`use lofty::probe::Probe;`
- 'Lofty' is the core library used to parse audio headers incredibly fast without decoding the audio stream.
`use rayon::prelude::*;`
- For multi-threading the harvest across hundreds of files simultaneously.
`use serde::Serialize;`
`use std::collections::HashMap;`
`use std::fs;`
`use std::io::{self, Write};`
`use std::path::{Path, PathBuf};`
`use std::sync::mpsc;`
`use std::thread;`
`use walkdir::WalkDir;`

Logic:
`pub struct TrackJson` & `pub struct PhysicsData`
- Structures that define the raw output of the harvest: paths, raw string maps of tags, and technical physics data like bitrates and mtimes.

`pub fn run(roots: Vec<PathBuf>, pretty: bool)`
- The CLI entry point for a direct harvest command.
- Scans paths for files, sets up a background printer thread, and uses Rayon to multithread the `harvest_file` function across all files, converting results to JSON and streaming them to standard output.

`fn scan_files(root: &Path, extensions: &[&str]) -> Vec<PathBuf>`
- A simple recursive search for files matching audio extensions.

`pub fn harvest_file(path: &Path) -> Result<TrackJson>`
- The core extraction engine for a single audio file.
- It reads the basic filesystem metadata (file size, mtime). It then uses `lofty::Probe` to detect the audio codec and reads the header properties to extract `PhysicsData` (duration, sample rate, bit depth). 
- To read user tags (like `TITLE` or `ARTIST`), it implements specific overrides for FLAC, Vorbis (Ogg), and Opus files. It manually parses the `vorbis_comments` block of these files to perfectly capture multi-value tags (e.g., merging multiple `GENRE` entries into a single string separated by `; `). If the file is an mp3/m4a or something else, it falls back to Lofty's generic tag reader. It returns the highly accurate raw tag map and physics data.
