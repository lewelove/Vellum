use serde::{Deserialize, Deserializer, Serialize};
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
    pub tracknumber: u32,
    #[serde(rename = "DISCNUMBER")]
    pub discnumber: u32,
    #[serde(default)]
    pub tags: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlbumInfo {
    #[serde(default)]
    pub album_path: String,
    #[serde(default)]
    pub unix_added: u64,
    #[serde(default)]
    pub date_added: String,
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
    pub manifests_mtime_sum: u64,
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
    #[serde(
        rename = "GENRE",
        default,
        deserialize_with = "deserialize_vec_or_string"
    )]
    pub genre: Vec<String>,
    #[serde(rename = "COMMENT", default)]
    pub comment: String,
    #[serde(rename = "ORIGINAL_DATE", default)]
    pub original_date: String,
    #[serde(rename = "ORIGINAL_YEAR", default)]
    pub original_year: String,
    #[serde(rename = "ORIGINAL_YYYY_MM", default)]
    pub original_yyyy_mm: String,
    #[serde(rename = "RELEASE_DATE", default)]
    pub release_date: String,
    #[serde(rename = "RELEASE_YEAR", default)]
    pub release_year: String,
    #[serde(rename = "RELEASE_YYYY_MM", default)]
    pub release_yyyy_mm: String,
    #[serde(default)]
    pub tags: HashMap<String, serde_json::Value>,
}

fn deserialize_vec_or_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum VecOrString {
        Vec(Vec<String>),
        String(String),
    }

    match VecOrString::deserialize(deserializer)? {
        VecOrString::Vec(v) => Ok(v),
        VecOrString::String(s) => Ok(s
            .split(';')
            .map(|part| part.trim().to_string())
            .filter(|part| !part.is_empty())
            .collect()),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockFile {
    pub album: AlbumLock,
    pub tracks: Vec<TrackLock>,
}
