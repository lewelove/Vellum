use serde::{Deserialize, Serialize};

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
