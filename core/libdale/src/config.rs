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
pub struct ActionDef {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct StorageConfig {
    pub music_directory: String,
    pub cache: String,
    pub state: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            music_directory: String::new(),
            cache: "~/.cache/dale".to_string(),
            state: "~/.local/share/dale".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ManifestConfig {
    pub audio_extensions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CompilerConfig {
    pub manifests: Option<Vec<String>>,
    pub audio_extensions: Option<Vec<String>>,
    pub jobs: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct CoversConfig {
    pub filter: String,
    pub size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct CoversRegistry {
    pub master: CoversConfig,
    pub targets: Vec<CoversConfig>,
}

impl Default for CoversRegistry {
    fn default() -> Self {
        Self {
            master: CoversConfig {
                filter: "mitchell".to_string(),
                size: 1080,
            },
            targets: Vec::new(),
        }
    }
}
