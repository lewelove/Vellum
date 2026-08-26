use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpacingConfig {
    pub x: f32,
    pub y: f32,
    pub top: f32,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            x: 20.0,
            y: 16.0,
            top: 20.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoverConfig {
    pub size: u32,
    pub filter: String,
}

impl Default for CoverConfig {
    fn default() -> Self {
        Self {
            size: 200,
            filter: "catmullrom".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextItemConfig {
    pub size: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextSpacingConfig {
    pub top: f32,
    pub middle: f32,
}

impl Default for TextSpacingConfig {
    fn default() -> Self {
        Self {
            top: 11.0,
            middle: 2.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextConfig {
    pub enable: bool,
    pub title: TextItemConfig,
    pub albumartist: TextItemConfig,
    pub spacing: TextSpacingConfig,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            enable: true,
            title: TextItemConfig { size: 15.0 },
            albumartist: TextItemConfig { size: 13.0 },
            spacing: TextSpacingConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AlbumCardConfig {
    pub cover: CoverConfig,
    pub text: TextConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AlbumGridConfig {
    pub spacing: SpacingConfig,
    pub album_card: AlbumCardConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub album_grid: AlbumGridConfig,
}
