use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackInfo {
    #[serde(default)]
    pub track_path: String,
    #[serde(default)]
    pub track_library_path: String,
    #[serde(default)]
    pub track_duration: u64,
    #[serde(default)]
    pub track_duration_time: String,
    #[serde(default)]
    pub encoding: String,
    #[serde(default)]
    pub sample_rate: u32,
    #[serde(default)]
    pub bits_per_sample: u8,
    #[serde(default)]
    pub channels: u8,
    #[serde(default)]
    pub track_mtime: u64,
    #[serde(default)]
    pub track_byte_size: u64,
    #[serde(default)]
    pub lyrics_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackLock {
    pub info: TrackInfo,
    #[serde(rename = "TITLE")]
    pub title: String,
    #[serde(rename = "ARTIST")]
    pub artist: String,
    #[serde(rename = "TRACKNUMBER")]
    pub tracknumber: String,
    #[serde(rename = "DISCNUMBER")]
    pub discnumber: String,
    #[serde(flatten)]
    pub keys: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlbumInfo {
    #[serde(default)]
    pub album_path: String,
    #[serde(default)]
    pub unix_added: u64,
    #[serde(default)]
    pub album_duration: u64,
    #[serde(default)]
    pub album_duration_time: String,
    #[serde(default)]
    pub total_discs: u32,
    #[serde(default)]
    pub total_tracks: u32,
    #[serde(default)]
    pub metadata_toml_hash: String,
    #[serde(default)]
    pub metadata_toml_mtime: u64,
    #[serde(default)]
    pub file_tag_subset_match: bool,
    #[serde(default)]
    pub cover_path: String,
    #[serde(default)]
    pub cover_hash: String,
    #[serde(default)]
    pub cover_mtime: u64,
    #[serde(default)]
    pub cover_byte_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlbumLock {
    pub info: AlbumInfo,
    #[serde(rename = "ALBUM")]
    pub album: String,
    #[serde(rename = "ALBUMARTIST")]
    pub albumartist: String,
    #[serde(rename = "DATE")]
    pub date: String,
    #[serde(flatten)]
    pub keys: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockFile {
    pub album: AlbumLock,
    pub tracks: Vec<TrackLock>,
    #[serde(default)]
    pub requires_python: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AlbumView {
    pub id: String,
    #[serde(flatten)]
    pub album_data: AlbumLock,
    pub tracks: Vec<TrackLock>,
}
