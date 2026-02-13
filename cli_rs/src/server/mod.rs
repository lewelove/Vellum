pub mod state;
pub mod library;
pub mod mpd;
pub mod api;

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

use crate::expand_path;
use self::state::{AppState, AppConfig};

pub async fn run(port: u16) -> Result<()> {
    let (root_dir, config_content) = find_config().context("Project root not found")?;
    let toml_val: toml::Value = toml::from_str(&config_content)?;
    
    let storage = toml_val.get("storage").context("config.toml missing [storage]")?;
    let lib_root_str = storage.get("library_root").and_then(|v| v.as_str()).unwrap_or(".");
    let thumb_root_str = storage.get("thumbnail_cache_folder").and_then(|v| v.as_str());

    let lib_path = expand_path(lib_root_str);
    let library_root = if lib_path.is_absolute() { lib_path } else { root_dir.join(lib_path) }
        .canonicalize().context("Invalid library_root")?;

    let app_config = Arc::new(AppConfig {
        library_root: library_root.clone(),
        thumbnail_root: thumb_root_str.map(expand_path),
    });

    let state_file = expand_path("~/.vellum/state.json");
    let ui_state_val = if state_file.exists() {
        let data = std::fs::read_to_string(&state_file).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({
            "activeTab": "home", "sortKey": "default", "sortOrder": "default",
            "groupKey": "genre", "filter": {"key": null, "val": null}
        })
    };

    let mut library = library::Library::new(library_root);
    library.scan().await;
    let library_arc = Arc::new(RwLock::new(library));
    let (tx, _) = broadcast::channel(100);

    let mpd_engine = mpd::start_actor(tx.clone(), Arc::clone(&library_arc), Arc::clone(&app_config));

    let app_state = Arc::new(AppState {
        library: library_arc,
        ui_state: RwLock::new(ui_state_val),
        tx,
        config: (*app_config).clone(),
        mpd_engine,
    });

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = api::router(Arc::clone(&app_state)).layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    log::info!("Vellum Server listening on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}

fn find_config() -> Result<(std::path::PathBuf, String)> {
    let mut curr = std::env::current_dir()?;
    loop {
        let conf_path = curr.join("config.toml");
        if conf_path.exists() {
            return Ok((curr, std::fs::read_to_string(&conf_path)?));
        }
        curr = curr.parent().context("config.toml not found")?.to_path_buf();
    }
}
