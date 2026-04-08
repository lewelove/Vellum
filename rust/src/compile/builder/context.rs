use crate::harvest::TrackJson;
use image::DynamicImage;
use serde_json::Value;
use std::path::Path;

pub struct AlbumContext<'a> {
    pub source: &'a Value,
    pub tracks: &'a [Value],
    pub album_root: &'a Path,
    pub library_root: &'a Path,
    pub meta_hash: &'a str,
    pub meta_mtime: u64,
    pub cover_hash: &'a str,
    pub cover_path: Option<&'a str>,
    pub cover_mtime: u64,
    pub cover_byte_size: u64,
    pub cover_image: Option<&'a DynamicImage>,
    pub config: &'a Value,
}

pub struct TrackContext<'a> {
    pub ordinal_track_number: u32,
    pub ordinal_disc_number: u32,
    pub harvest: &'a TrackJson,
    pub source: &'a Value,
    pub album_source: &'a Value,
    pub album_root: &'a Path,
    pub library_root: &'a Path,
}
