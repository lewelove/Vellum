use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AppConfig {
    pub storage: StorageConfig,
    pub generate: Option<GenerateConfig>,
    pub compress: Option<CompressConfig>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StorageConfig {
    pub library_root: String,
    pub library_export: Option<String>,
    pub thumbnail_cache_folder: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GenerateConfig {
    #[serde(default)]
    pub supported_extensions: Vec<String>,
    #[serde(default)]
    pub grouping_keys: Vec<String>,
    #[serde(default = "default_separator")]
    pub naming_separator: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CompressConfig {
    pub album: Option<LayoutConfig>,
    pub tracks: Option<LayoutConfig>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LayoutConfig {
    #[serde(default)]
    pub layout: Vec<LayoutItem>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LayoutItem {
    Key(String),
    Block(HashMap<String, Vec<String>>),
}

fn default_separator() -> String {
    "_".to_string()
}
