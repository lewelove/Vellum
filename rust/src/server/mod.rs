pub mod api;
pub mod library;
pub mod mpd;
pub mod state;
pub mod watchdog;

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tower_http::cors::{Any, CorsLayer};

use self::state::{AppConfig as ServerConfig, AppState};
use crate::config::AppConfig;
use crate::expand_path;

pub async fn run(port: u16) -> Result<()> {
    let (config, _, config_path) = AppConfig::load().context("Failed to load application configuration")?;
    let config_dir = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();

    let lib_root_str = &config.storage.library_root;
    let thumb_root_str = config.storage.thumbnail_cache_folder.as_deref();
    let thumb_size = config.theme.as_ref().map_or(200, |t| t.thumbnail_size);

    let library_root = expand_path(lib_root_str)
        .canonicalize()
        .context("Invalid library_root path")?;

    let shader_cfg = config.theme.as_ref().and_then(|t| t.shader.clone());
    let mut resolved_path = None;
    if let Some(ref s) = shader_cfg 
        && let Some(ref p) = s.path 
    {
        let p_buf = PathBuf::from(p);
        resolved_path = if p_buf.is_absolute() {
            Some(p_buf)
        } else {
            Some(config_dir.join(p_buf))
        };
    }

    let css_path = config_dir.join("vellum.css");
    let resolved_css_path = if css_path.exists() {
        Some(css_path)
    } else {
        None
    };

    let facets_path = config_dir.join("facets.js");
    let resolved_facets_path = if facets_path.exists() {
        Some(facets_path)
    } else {
        None
    };

    let sorters_path = config_dir.join("sorters.js");
    let resolved_sorters_path = if sorters_path.exists() {
        Some(sorters_path)
    } else {
        None
    };

    let server_config = ServerConfig {
        library_root: library_root.clone(),
        thumbnail_root: thumb_root_str.map(expand_path),
        thumbnail_size: thumb_size,
        shader: shader_cfg,
        resolved_shader_path: resolved_path,
        resolved_css_path,
        resolved_facets_path,
        resolved_sorters_path,
    };

    let state_file = expand_path("~/.vellum/state.json");
    let ui_state_val = if state_file.exists() {
        let data = std::fs::read_to_string(&state_file).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({
            "activeTab": "home",
            "sortKey": "default",
            "sortOrder": "default",
            "groupKey": "genre",
            "filter": {
                "key": null,
                "val": null
            },
            "queuePanels": {
                "lyrics": false,
                "tracks": true
            }
        })
    };

    let mut library = library::Library::new(library_root);
    library.scan();
    let library_arc = Arc::new(RwLock::new(library));
    let (tx, _) = broadcast::channel(100);

    let mpd_engine = mpd::start_actor(
        tx.clone(),
        Arc::clone(&library_arc),
        Arc::new(server_config.clone()),
    );

    let app_state = Arc::new(AppState {
        library: library_arc,
        ui_state: RwLock::new(ui_state_val),
        tx,
        config: RwLock::new(server_config),
        mpd_engine,
    });

    watchdog::start(config_path, Arc::clone(&app_state));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = api::router(Arc::clone(&app_state)).layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    log::info!("Vellum Server listening on http://{addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}
