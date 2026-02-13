mod api;
mod library;
mod mpd_engine;

use anyhow::{Context, Result};
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tower_http::cors::{Any, CorsLayer};
use std::path::PathBuf;

use crate::expand_path;

pub struct AppState {
    pub library: RwLock<library::Library>,
    pub ui_state: RwLock<serde_json::Value>,
    pub tx: broadcast::Sender<String>,
    pub config: AppConfig,
}

#[derive(Clone)]
pub struct AppConfig {
    pub library_root: PathBuf,
    pub thumbnail_root: Option<PathBuf>,
}

fn find_config() -> Result<(PathBuf, String)> {
    let mut curr = std::env::current_dir()?;
    loop {
        let conf_path = curr.join("config.toml");
        if conf_path.exists() {
            let content = std::fs::read_to_string(&conf_path)?;
            return Ok((curr, content));
        }
        if let Some(parent) = curr.parent() {
            curr = parent.to_path_buf();
        } else {
            break;
        }
    }
    anyhow::bail!("config.toml not found in current or parent directories")
}

pub async fn run(port: u16) -> Result<()> {
    log::info!("Starting Vellum Server (Rust) on port {}", port);

    // 1. Load and Parse Configuration
    let (root_dir, config_content) = find_config().context("Failed to locate project root/config.toml")?;
    
    // CHANGE: Use toml::from_str for document parsing instead of .parse()
    let toml_val: toml::Value = toml::from_str(&config_content).context("Failed to parse config.toml")?;
    
    let storage = toml_val.get("storage");
    let lib_root_str = storage.and_then(|v| v.get("library_root")).and_then(|v| v.as_str()).unwrap_or(".");
    let thumb_root_str = storage.and_then(|v| v.get("thumbnail_cache_folder")).and_then(|v| v.as_str());

    // Ensure library root is absolute relative to the project root
    let lib_path = expand_path(lib_root_str);
    let library_root = if lib_path.is_absolute() {
        lib_path
    } else {
        root_dir.join(lib_path)
    }.canonicalize().context("Invalid library_root path")?;

    let app_config = AppConfig {
        library_root,
        thumbnail_root: thumb_root_str.map(|s| expand_path(s)),
    };

    // 2. Load UI State
    let state_dir = expand_path("~/.vellum");
    let state_file = state_dir.join("state.json");
    let ui_state_val = if state_file.exists() {
        let data = std::fs::read_to_string(&state_file).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({
            "activeTab": "home",
            "sortKey": "default",
            "sortOrder": "default",
            "groupKey": "genre",
            "filter": {"key": null, "val": null}
        })
    };

    // 3. Initialize Library
    let mut library = library::Library::new(app_config.library_root.clone());
    library.scan().await;

    // 4. Setup Broadcast Channel
    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState {
        library: RwLock::new(library),
        ui_state: RwLock::new(ui_state_val),
        tx: tx.clone(),
        config: app_config,
    });

    // 5. Start MPD Engine
    mpd_engine::start_monitor(app_state.clone());

    // 6. Setup Router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(api::ws_handler))
        .route("/api/state", post(api::update_state))
        .route("/api/internal/reset", post(api::trigger_full_reset))
        .route("/api/internal/reload", post(api::trigger_reload))
        .route("/api/covers/:hash", get(api::get_cover_thumbnail))
        .route("/api/assets/:id/cover", get(api::get_album_cover))
        .route("/api/play/:id", post(api::play_album))
        .route("/api/play-disc/:id", post(api::play_disc))
        .route("/api/queue/:id", post(api::queue_album))
        .route("/api/open/:id", post(api::open_album_folder))
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("Listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
