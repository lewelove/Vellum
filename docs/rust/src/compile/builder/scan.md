File: rust/src/compile/builder/scan.rs
Role: Filesystem Scanner

Description:
This file contains utility functions to aggressively scan directories for valid albums (identified by the presence of a sidecar metadata file) and to collect valid audio files within an album directory.

Imports:
`use anyhow::Result;`
- Used for error handling.
`use std::path::{Path, PathBuf};`
- Used for path manipulation.
`use walkdir::WalkDir;`
- Used to recursively traverse directory trees.

Logic:
`pub fn find_target_albums(path: &Path, max_depth: usize) -> Vec<PathBuf>`
- Recursively searches a path to find directories acting as albums.
- The logic checks if the provided path directly contains a `metadata.toml` file. If it does, the search stops and returns that path. Otherwise, it uses `WalkDir` to traverse subdirectories up to a specified depth. Any directory containing a `metadata.toml` is considered an album and is added to the result vector.

`pub fn scan_audio_files(root: &Path, extensions: &[&str]) -> Vec<PathBuf>`
- Collects all valid audio files within an album directory and sorts them.
- It walks the album directory tree (up to a shallow depth of 3) looking for files. It checks each file's extension against a list of supported audio extensions (like `.flac` or `.mp3`). All matching files are collected and then sorted using an "alphanumeric" sorter. This ensures that files named `1 Track.flac` and `10 Track.flac` sort logically rather than strictly alphabetically, matching human expectations.
