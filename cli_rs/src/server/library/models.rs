use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Clone, Debug, Serialize)]
pub struct AlbumView {
    pub id: String,
    #[serde(flatten)]
    pub album_data: AlbumLock,
    pub tracks: Vec<TrackLock>,
}
