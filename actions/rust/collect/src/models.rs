pub struct FormattingConfig {
    pub album: String,
    pub info: String,
}

pub struct Track {
    pub discnumber: u32,
    pub tracknumber: u32,
    pub title: String,
    pub artist: Option<String>,
}

#[derive(Default)]
pub struct AlbumData {
    pub albumartist: String,
    pub album: String,
    pub date: String,
    pub tracks: Vec<Track>,
    pub discogs_master_raw: Option<serde_json::Value>,
    pub discogs_release_raw: Option<serde_json::Value>,
    pub musicbrainz_release_raw: Option<serde_json::Value>,
    pub musicbrainz_releasegroup_raw: Option<serde_json::Value>,
    pub musicbrainz_all_releases_raw: Option<serde_json::Value>,
}
