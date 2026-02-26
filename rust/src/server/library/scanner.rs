use crate::server::library::models::{AlbumView, LockFile};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Library {
    pub root: PathBuf,
    pub albums: Vec<AlbumView>,
    pub album_map: HashMap<String, AlbumView>,
    pub track_map: HashMap<String, PathBuf>,
    pub path_lookup: HashMap<String, String>,
}

impl Library {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            albums: Vec::new(),
            album_map: HashMap::new(),
            track_map: HashMap::new(),
            path_lookup: HashMap::new(),
        }
    }

    fn normalize_path(path: &str) -> String {
        path.trim_start_matches('/').to_string()
    }

    pub fn scan(&mut self) {
        log::info!("Scanning Library at {}", self.root.display());

        let mut albums = Vec::new();
        let mut album_map = HashMap::new();
        let mut track_map = HashMap::new();
        let mut path_lookup = HashMap::new();

        let entries: Vec<PathBuf> = WalkDir::new(&self.root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_name() == "metadata.lock.json")
            .map(|e| e.path().to_path_buf())
            .collect();

        for lock_path in entries {
            match std::fs::read_to_string(&lock_path) {
                Ok(content) => match serde_json::from_str::<LockFile>(&content) {
                    Ok(lock_data) => {
                        let alb_id = lock_data.album.info.album_path.clone();
                        let album_dir = lock_path.parent().unwrap_or(&self.root);

                        let view = AlbumView {
                            id: alb_id.clone(),
                            album_data: lock_data.album.clone(),
                            tracks: lock_data.tracks.clone(),
                        };

                        albums.push(view.clone());
                        album_map.insert(alb_id.clone(), view);

                        for track in &lock_data.tracks {
                            let t_id = track.info.track_library_path.clone();
                            let abs_path = album_dir.join(&track.info.track_path);
                            track_map.insert(t_id, abs_path);

                            let full_rel_path = Path::new(&alb_id).join(&track.info.track_path);
                            let normalized =
                                Self::normalize_path(full_rel_path.to_str().unwrap_or(""));
                            path_lookup.insert(normalized, alb_id.clone());
                        }
                    }
                    Err(e) => {
                        log::error!("Schema Mismatch at {}: {e}", lock_path.display());
                    }
                },
                Err(e) => {
                    log::error!("Failed to read lock file at {}: {e}", lock_path.display());
                }
            }
        }

        albums.sort_by(|a, b| a.id.cmp(&b.id));
        self.albums = albums;
        self.album_map = album_map;
        self.track_map = track_map;
        self.path_lookup = path_lookup;

        log::info!("Library Initialized: {} albums.", self.albums.len());
    }

    pub fn update_album(&mut self, folder_path_str: &str) -> Option<AlbumView> {
        let lock_path = Path::new(folder_path_str).join("metadata.lock.json");
        if let Ok(content) = std::fs::read_to_string(&lock_path) {
            match serde_json::from_str::<LockFile>(&content) {
                Ok(lock_data) => {
                    let alb_id = lock_data.album.info.album_path.clone();
                    let album_dir = lock_path.parent().unwrap_or(&self.root);

                    let view = AlbumView {
                        id: alb_id.clone(),
                        album_data: lock_data.album.clone(),
                        tracks: lock_data.tracks.clone(),
                    };

                    self.album_map.insert(alb_id.clone(), view.clone());
                    if let Some(idx) = self.albums.iter().position(|x| x.id == alb_id) {
                        self.albums[idx] = view.clone();
                    } else {
                        self.albums.push(view.clone());
                        self.albums.sort_by(|a, b| a.id.cmp(&b.id));
                    }

                    for track in &lock_data.tracks {
                        let t_id = track.info.track_library_path.clone();
                        let abs_path = album_dir.join(&track.info.track_path);
                        self.track_map.insert(t_id, abs_path);

                        let full_rel_path = Path::new(&alb_id).join(&track.info.track_path);
                        let normalized = Self::normalize_path(full_rel_path.to_str().unwrap_or(""));
                        self.path_lookup.insert(normalized, alb_id.clone());
                    }
                    return Some(view);
                }
                Err(e) => {
                    log::error!(
                        "Schema Mismatch during update at {}: {e}",
                        lock_path.display()
                    );
                }
            }
        }
        None
    }
}
