use crate::server::logic::LogicEngine;
use crate::server::mpd::MpdEngine;
use libdale::lua::config::{ActionConfig, CoversRegistry, InterfaceConfig};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tokio::sync::{broadcast, RwLock};

pub struct AppState {
    pub logic: Arc<RwLock<LogicEngine>>,
    pub ui_state: RwLock<serde_json::Value>,
    pub tx: broadcast::Sender<String>,
    pub config: RwLock<AppConfig>,
    pub mpd_engine: MpdEngine,
    pub tracked_albums: Arc<Mutex<HashSet<String>>>,
    pub full_rescan_needed: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct AppConfig {
    pub library_root: PathBuf,
    pub cache_root: PathBuf,
    pub state_root: PathBuf,
    pub resolved_dependencies: Vec<PathBuf>,
    pub covers: CoversRegistry,
    pub interfaces: HashMap<String, InterfaceConfig>,
    pub actions: HashMap<String, ActionConfig>,
    pub config_dir: PathBuf,
    pub app: libdale::lua::config::AppConfig,
}
