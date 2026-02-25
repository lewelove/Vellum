use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackInfo {
    pub track_path: String,
    pub track_library_path: String,
    pub track_duration: u64,
    pub track_duration_time: String,
    pub encoding: String,
    pub sample_rate: u32,
    pub bits_per_sample: u8,
    pub channels: u8,
    pub track_mtime: u64,
    pub track_byte_size: u64,
    pub lyrics_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackLock {
    pub info: TrackInfo,
    #[serde(flatten)]
    pub keys: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlbumInfo {
    pub album_path: String,
    pub unix_added: u64,
    pub album_duration: u64,
    pub album_duration_time: String,
    pub total_discs: u32,
    pub total_tracks: u32,
    pub metadata_toml_hash: String,
    pub metadata_toml_mtime: u64,
    pub file_tag_subset_match: bool,
    pub cover_path: String,
    pub cover_hash: String,
    pub cover_mtime: u64,
    pub cover_byte_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlbumLock {
    pub info: AlbumInfo,
    #[serde(flatten)]
    pub keys: HashMap<String, serde_json::Value>,
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
