use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::http::{StatusCode, header};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use std::sync::Arc;
use crate::server::state::AppState;

pub async fn get_cover_thumbnail(
    Path(hash): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if let Some(root) = &state.config.thumbnail_root {
        let path = root.join(format!("{}.png", hash));
        return serve_image(path).await;
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn get_album_cover(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path_opt = {
        let lib = state.library.read().await;
        lib.album_map.get(&id).and_then(|a| {
            let cp = &a.album_data.info.cover_path;
            Some(state.config.library_root.join(&id).join(cp))
        })
    };

    if let Some(path) = path_opt {
        return serve_image(path).await;
    }
    StatusCode::NOT_FOUND.into_response()
}

async fn serve_image(path: PathBuf) -> Response {
    if let Ok(mut file) = File::open(&path).await {
        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).await.is_ok() {
            let mime = if path.extension().map_or(false, |e| e == "png") { "image/png" } else { "image/jpeg" };
            return (
                [(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "public, max-age=31536000, immutable")],
                buf
            ).into_response();
        }
    }
    StatusCode::NOT_FOUND.into_response()
}
