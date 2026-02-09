use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AppConfig {
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StorageConfig {
    pub library_root: String,
    pub library_export: Option<String>,
    pub thumbnail_cache_folder: Option<String>,
}
