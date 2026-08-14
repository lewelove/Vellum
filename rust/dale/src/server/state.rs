use crate::server::logic::LogicEngine;
use crate::server::mpd::MpdEngine;
use libdale::config::{ActionConfig, CoversRegistry, InterfaceConfig};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tokio::sync::{broadcast, RwLock};

#[derive(Default)]
pub struct DependencyGraph {
    pub file_to_albums: HashMap<PathBuf, HashSet<PathBuf>>,
    pub album_to_files: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl DependencyGraph {
    pub fn update_album_deps(&mut self, album_path: PathBuf, new_deps: HashSet<PathBuf>) {
        if let Some(old_deps) = self.album_to_files.remove(&album_path) {
            for dep in old_deps {
                if let Some(albums) = self.file_to_albums.get_mut(&dep) {
                    albums.remove(&album_path);
                    if albums.is_empty() {
                        self.file_to_albums.remove(&dep);
                    }
                }
            }
        }

        for dep in &new_deps {
            self.file_to_albums
                .entry(dep.clone())
                .or_default()
                .insert(album_path.clone());
        }
        self.album_to_files.insert(album_path, new_deps);
    }

    pub fn remove_album(&mut self, album_path: &Path) {
        if let Some(deps) = self.album_to_files.remove(album_path) {
            for dep in deps {
                if let Some(albums) = self.file_to_albums.get_mut(&dep) {
                    albums.remove(album_path);
                    if albums.is_empty() {
                        self.file_to_albums.remove(&dep);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateScope {
    All,
    Paths(HashSet<PathBuf>),
}

impl UpdateScope {
    pub fn merge(&mut self, other: Self) {
        match (self, other) {
            (Self::All, _) => {}
            (this, Self::All) => *this = Self::All,
            (Self::Paths(set_a), Self::Paths(set_b)) => {
                set_a.extend(set_b);
            }
        }
    }
}

#[derive(Default)]
pub struct UpdateState {
    pub is_running: bool,
    pub pending: Option<PendingUpdateParams>,
}

pub struct PendingUpdateParams {
    pub scope: UpdateScope,
    pub force: bool,
    pub jobs: Option<usize>,
    pub silent: bool,
    pub log_txs: Vec<tokio::sync::mpsc::Sender<String>>,
}

pub struct AppState {
    pub logic: Arc<RwLock<LogicEngine>>,
    pub ui_state: RwLock<serde_json::Value>,
    pub tx: broadcast::Sender<String>,
    pub config: RwLock<AppConfig>,
    pub mpd_engine: MpdEngine,
    pub tracked_albums: Arc<Mutex<HashSet<String>>>,
    pub full_rescan_needed: Arc<AtomicBool>,
    pub active_writes: Arc<Mutex<HashSet<PathBuf>>>,
    pub update_lock: Arc<Mutex<UpdateState>>,
    pub deps_graph: Arc<RwLock<DependencyGraph>>,
}

#[derive(Clone)]
pub struct AppConfig {
    pub music_directory: PathBuf,
    pub cache_root: PathBuf,
    pub state_root: PathBuf,
    pub resolved_dependencies: Vec<PathBuf>,
    pub covers: CoversRegistry,
    pub interfaces: HashMap<String, InterfaceConfig>,
    pub actions: HashMap<String, ActionConfig>,
    pub config_dir: PathBuf,
    pub app: libdale::config::AppConfig,
}
