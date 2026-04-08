use crate::server::library::Library;
use crate::server::mpd::MpdEngine;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

pub struct AppState {
    pub library: Arc<RwLock<Library>>,
    pub ui_state: RwLock<serde_json::Value>,
    pub tx: broadcast::Sender<String>,
    pub config: RwLock<AppConfig>,
    pub mpd_engine: MpdEngine,
}

#[derive(Clone)]
pub struct AppConfig {
    pub library_root: PathBuf,
    pub thumbnail_root: Option<PathBuf>,
    pub thumbnail_size: u32,
    pub shader: Option<crate::config::ShaderConfig>,
    pub resolved_shader_path: Option<PathBuf>,
    pub resolved_css_path: Option<PathBuf>,
    pub resolved_facets_path: Option<PathBuf>,
    pub resolved_sorters_path: Option<PathBuf>,
}
