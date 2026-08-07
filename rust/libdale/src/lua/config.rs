use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub manifest: ManifestConfig,
    #[serde(default)]
    pub compiler: CompilerConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct InterfaceConfig {
    #[serde(default)]
    pub enable: bool,
    pub directory: Option<String>,
    pub run: Option<String>,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub assets: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ActionConfig {
    pub run: Option<String>,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    #[serde(default)]
    pub library: String,
    pub environment: Option<String>,
    #[serde(default = "default_cache")]
    pub cache: String,
    #[serde(default = "default_state")]
    pub state: String,
}

fn default_cache() -> String {
    "~/.cache/dale".to_string()
}
fn default_state() -> String {
    "~/.local/share/dale".to_string()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            library: String::new(),
            environment: None,
            cache: default_cache(),
            state: default_state(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ManifestConfig {
    pub audio_files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CompilerConfig {
    pub manifests: Option<Vec<String>>,
    pub jobs: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct CoversConfig {
    pub filter: String,
    pub size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub struct CoversRegistry {
    #[serde(default = "default_master_cover")]
    pub master: CoversConfig,
    #[serde(default)]
    pub targets: Vec<CoversConfig>,
}

fn default_master_cover() -> CoversConfig {
    CoversConfig {
        filter: "mitchell".to_string(),
        size: 1080,
    }
}
