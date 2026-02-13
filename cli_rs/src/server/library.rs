use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackLock {
    pub track_path: String,
    pub track_library_path: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlbumLock {
    pub album_root_path: Option<String>,
    pub cover_path: Option<String>,
    pub cover_hash: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockFile {
    pub album: AlbumLock,
    pub tracks: Vec<TrackLock>,
}

// The structure we send to the UI
#[derive(Clone, Debug, Serialize)]
pub struct AlbumView {
    pub id: String,
    #[serde(flatten)]
    pub album_data: AlbumLock,
    pub tracks: Vec<TrackLock>,
}

pub struct Library {
    pub root: PathBuf,
    pub albums: Vec<AlbumView>,
    pub album_map: HashMap<String, AlbumView>,
    pub track_map: HashMap<String, PathBuf>, // track_library_path -> absolute path
    pub path_lookup: HashMap<String, String>, // normalized relative path -> album_id
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

    fn normalize_path(&self, path: &str) -> String {
        path.trim_start_matches('/').to_string()
    }

    pub async fn scan(&mut self) {
        log::info!("Scanning library at {:?}", self.root);
        
        let mut albums = Vec::new();
        let mut album_map = HashMap::new();
        let mut track_map = HashMap::new();
        let mut path_lookup = HashMap::new();

        let walker = WalkDir::new(&self.root).into_iter();

        // Collect valid lock files first to process
        let entries: Vec<PathBuf> = walker
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "metadata.lock.json")
            .map(|e| e.path().to_path_buf())
            .collect();

        for lock_path in entries {
            if let Ok(content) = std::fs::read_to_string(&lock_path) {
                if let Ok(lock_data) = serde_json::from_str::<LockFile>(&content) {
                    if let Some(alb_id) = &lock_data.album.album_root_path {
                        let album_dir = lock_path.parent().unwrap_or(&self.root);
                        
                        let view = AlbumView {
                            id: alb_id.clone(),
                            album_data: lock_data.album.clone(),
                            tracks: lock_data.tracks.clone(),
                        };

                        albums.push(view.clone());
                        album_map.insert(alb_id.clone(), view);

                        for track in &lock_data.tracks {
                            if let Some(t_id) = &track.track_library_path {
                                let abs_path = album_dir.join(&track.track_path);
                                track_map.insert(t_id.clone(), abs_path);
                            }
                            
                            // Path lookup logic
                            let full_rel_path = Path::new(alb_id).join(&track.track_path);
                            let normalized = self.normalize_path(full_rel_path.to_str().unwrap_or(""));
                            path_lookup.insert(normalized, alb_id.clone());
                        }
                    }
                }
            }
        }

        // Sort by ID to keep order stable
        albums.sort_by(|a, b| a.id.cmp(&b.id));

        self.albums = albums;
        self.album_map = album_map;
        self.track_map = track_map;
        self.path_lookup = path_lookup;

        log::info!("Library Initialized: {} albums.", self.albums.len());
    }

    pub fn update_album(&mut self, folder_path_str: &str) -> Option<AlbumView> {
        let lock_path = Path::new(folder_path_str).join("metadata.lock.json");
        if !lock_path.exists() {
            return None;
        }

        if let Ok(content) = std::fs::read_to_string(&lock_path) {
            if let Ok(lock_data) = serde_json::from_str::<LockFile>(&content) {
                if let Some(alb_id) = &lock_data.album.album_root_path {
                     let album_dir = lock_path.parent().unwrap_or(&self.root);

                    let view = AlbumView {
                        id: alb_id.clone(),
                        album_data: lock_data.album.clone(),
                        tracks: lock_data.tracks.clone(),
                    };

                    // Update Maps
                    self.album_map.insert(alb_id.clone(), view.clone());
                    
                    // Update list (find and replace or append)
                    if let Some(idx) = self.albums.iter().position(|x| x.id == *alb_id) {
                        self.albums[idx] = view.clone();
                    } else {
                        self.albums.push(view.clone());
                        // Maintain sort
                        self.albums.sort_by(|a, b| a.id.cmp(&b.id));
                    }

                    for track in &lock_data.tracks {
                        if let Some(t_id) = &track.track_library_path {
                             let abs_path = album_dir.join(&track.track_path);
                             self.track_map.insert(t_id.clone(), abs_path);
                        }
                        
                        let full_rel_path = Path::new(alb_id).join(&track.track_path);
                        let normalized = self.normalize_path(full_rel_path.to_str().unwrap_or(""));
                        self.path_lookup.insert(normalized, alb_id.clone());
                    }

                    return Some(view);
                }
            }
        }
        None
    }
}
