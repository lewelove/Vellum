use anyhow::Result;
use walkdir::WalkDir;
use std::path::{Path, PathBuf};

pub fn scan_files(root: &Path, extensions: &[String]) -> Result<Vec<PathBuf>> {
    let ext_set: Vec<String> = extensions.iter()
        .map(|e| e.trim_start_matches('.').to_lowercase())
        .collect();

    let mut files = Vec::new();

    for entry in WalkDir::new(root).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if ext_set.contains(&ext_str.to_lowercase()) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }
    }

    Ok(files)
}
