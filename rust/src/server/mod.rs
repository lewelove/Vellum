pub mod state;
pub mod library;
pub mod mpd;
pub mod api;

use anyhow::{
    Context,
    Result,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{
    broadcast,
    RwLock,
};
use tower_http::cors::{
    Any,
    CorsLayer,
};

use crate::expand_path;
use crate::config::AppConfig;
use self::state::{
    AppState,
    AppConfig as ServerConfig,
};

pub async fn run(port: u16) -> Result<()> {
    let (config, _, _) = AppConfig::load().context("Failed to load application configuration")?;
    
    let lib_root_str = &config.storage.library_root;
    let thumb_root_str = config.storage.thumbnail_cache_folder.as_deref();

    let library_root = expand_path(lib_root_str)
        .canonicalize()
        .context("Invalid library_root path")?;

    let server_config = Arc::new(ServerConfig {
        library_root: library_root.clone(),
        thumbnail_root: thumb_root_str.map(expand_path),
    });

    let state_file = expand_path("~/.vellum/state.json");
    let ui_state_val = if state_file.exists() {
        let data = std::fs::read_to_string(&state_file).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({
            "activeTab": "home",
            "sortKey": "default",
            "sortOrder": "default",
            "groupKey": "genre",
            "filter": {
                "key": null,
                "val": null
            }
        })
    };

    let mut library = library::Library::new(library_root);
    library.scan().await;
    let library_arc = Arc::new(RwLock::new(library));
    let (tx, _) = broadcast::channel(100);

    let mpd_engine = mpd::start_actor(
        tx.clone(),
        Arc::clone(&library_arc),
        Arc::clone(&server_config),
    );

    let app_state = Arc::new(AppState {
        library: library_arc,
        ui_state: RwLock::new(ui_state_val),
        tx,
        config: (*server_config).clone(),
        mpd_engine,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
        
    let app = api::router(Arc::clone(&app_state)).layer(cors);

    let addr = SocketAddr::from((
        [127, 0, 0, 1],
        port
    ));
    log::info!("Vellum Server listening on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}
