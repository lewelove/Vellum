use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_target_albums(path: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if path.join("metadata.toml").exists() {
        results.push(
            path.to_path_buf()
        );
    } else {
        for entry in WalkDir::new(path).max_depth(max_depth).into_iter().filter_map(Result::ok) {
            if entry.file_name() == "metadata.toml"
                && let Some(parent) = entry.path().parent() {
                results.push(
                    parent.to_path_buf()
                );
            }
        }
    }
    results
}

pub fn scan_audio_files(root: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let max_audio_depth = 3; 
    
    let mut files: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_audio_depth)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| extensions.contains(&format!(".{}", ext.to_lowercase()).as_str()))
        })
        .collect();

    files.sort_by(|a, b| alphanumeric_sort::compare_path(a, b));
    files
}
