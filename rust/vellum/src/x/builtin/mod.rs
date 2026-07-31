pub mod open_folder;
pub mod open_lock;
pub mod open_manifest;
pub mod open_terminal;

use anyhow::Result;
use std::path::Path;

pub fn execute_builtin(name: &str, album_path: &Path, config: &serde_json::Value) -> Result<bool> {
    match name {
        "open_folder" | "open" => {
            open_folder::run(album_path)?;
            Ok(true)
        }
        "open_manifest" => {
            open_manifest::run(album_path)?;
            Ok(true)
        }
        "open_lock" => {
            open_lock::run(album_path)?;
            Ok(true)
        }
        "open_terminal" => {
            open_terminal::run(album_path, config)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
