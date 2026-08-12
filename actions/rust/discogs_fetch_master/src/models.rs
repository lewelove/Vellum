use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ActionPayload {
    pub albums: Vec<serde_json::Value>,
    #[serde(default)]
    pub config: ConfigBlock,
    #[serde(default)]
    pub options: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct ConfigBlock {
    #[serde(default)]
    pub dale: DaleConfig,
    #[serde(default)]
    pub action: ActionConfig,
}

#[derive(Deserialize, Debug, Default)]
pub struct DaleConfig {
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Deserialize, Debug, Default)]
pub struct StorageConfig {
    #[serde(default)]
    pub music_directory: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct ActionConfig {
    #[serde(default = "default_info_dir")]
    pub info_dir: String,
    #[serde(default = "default_filename")]
    pub filename: String,
}

fn default_info_dir() -> String {
    "Info".to_string()
}

fn default_filename() -> String {
    "discogs_master.json".to_string()
}

#[derive(Deserialize, Debug, Clone)]
pub struct DiscogsSearchResult {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub year: Option<serde_json::Value>,
    #[serde(default)]
    pub genre: Vec<String>,
    #[serde(default)]
    pub style: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct DiscogsSearchResponse {
    #[serde(default)]
    pub results: Vec<DiscogsSearchResult>,
}

pub enum UserSelection {
    Selected(usize),
    Skip,
    Quit,
}
