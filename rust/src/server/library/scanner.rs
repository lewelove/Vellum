use crate::server::library::models::LockFile;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Library {
    pub root: PathBuf,
}

impl Library {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn scan(&self, query_engine: &mut crate::server::query::QueryEngine) {
        log::info!("Scanning Library at {}", self.root.display());

        let entries: Vec<PathBuf> = WalkDir::new(&self.root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_name() == "metadata.lock.json")
            .map(|e| e.path().to_path_buf())
            .collect();

        let _ = query_engine.clear();

        for lock_path in entries {
            if let Ok(content) = std::fs::read_to_string(&lock_path) {
                if let Ok(lock_data) = serde_json::from_str::<LockFile>(&content) {
                    let alb_id = lock_data.album.info.album_path;
                    let _ = query_engine.ingest(&alb_id, &content);
                }
            }
        }

        if let Err(e) = query_engine.build_cache() {
            log::error!("Failed to build query cache: {}", e);
        }

        log::info!("Library Query Engine Initialized.");
    }

    pub fn update_album(&self, folder_path_str: &str, query_engine: &mut crate::server::query::QueryEngine) {
        let lock_path = Path::new(folder_path_str).join("metadata.lock.json");
        if let Ok(content) = std::fs::read_to_string(&lock_path) {
            if let Ok(lock_data) = serde_json::from_str::<LockFile>(&content) {
                let alb_id = lock_data.album.info.album_path;
                let _ = query_engine.remove_album(&alb_id);
                let _ = query_engine.ingest(&alb_id, &content);
                let _ = query_engine.build_cache();
            }
        }
    }
}
