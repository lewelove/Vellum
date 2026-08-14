use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ActionPayload {
    #[serde(default)]
    pub albums: Vec<serde_json::Value>,
    #[serde(default)]
    pub config: ConfigBlock,
}

#[derive(Deserialize, Default)]
pub struct ConfigBlock {
    #[serde(default)]
    pub dale: DaleConfig,
}

#[derive(Deserialize, Default)]
pub struct DaleConfig {
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Deserialize, Default)]
pub struct StorageConfig {
    #[serde(default)]
    pub music_directory: String,
    #[serde(default)]
    pub cache: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct FileStat {
    pub mtime: u64,
    pub size: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct CoverFileInfo {
    pub path: String,
    pub mtime: u64,
    pub byte_size: u64,
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct CoverMetricsDoc {
    pub cover: CoverFileInfo,
    pub chroma: f64,
    pub entropy: usize,
}
