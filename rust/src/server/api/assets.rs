use crate::server::state::AppState;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn get_cover_thumbnail(
    Path((size, hash)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let root_opt = {
        let guard = state.config.read().await;
        guard.thumbnail_root.clone()
    };
    
    if let Some(root) = root_opt {
        let path = root.join(&size).join(format!("{hash}.png"));

        match serve_image(path.clone()).await {
            resp if resp.status() == StatusCode::OK => resp,
            _ => {
                log::error!("FS 404: File not found at -> {}", path.display());
                StatusCode::NOT_FOUND.into_response()
            }
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn get_album_cover(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path_opt = {
        let lib = state.library.read().await;
        let config_guard = state.config.read().await;
        lib.album_map.get(&id).map(|a| {
            let cp = &a.album_data.info.cover_path;
            config_guard.library_root.join(&id).join(cp)
        })
    };

    if let Some(path) = path_opt {
        return serve_image(path).await;
    }
    StatusCode::NOT_FOUND.into_response()
}

pub async fn get_lyrics(
    Path((id, path)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let full_path = {
        let config_guard = state.config.read().await;
        config_guard.library_root.join(&id).join(&path)
    };

    if full_path.exists()
        && full_path.is_file()
        && let Ok(mut file) = File::open(&full_path).await
    {
        let mut buf = String::new();
        if file.read_to_string(&mut buf).await.is_ok() {
            return (
                [
                    (
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("text/plain; charset=utf-8"),
                    ),
                    (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
                ],
                buf,
            )
                .into_response();
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

pub async fn get_custom_shader(State(state): State<Arc<AppState>>) -> Response {
    let path_opt = {
        let guard = state.config.read().await;
        guard.resolved_shader_path.clone()
    };

    if let Some(path) = path_opt {
        if let Ok(mut file) = File::open(&path).await {
            let mut buf = String::new();
            if file.read_to_string(&mut buf).await.is_ok() {
                return (
                    [
                        (
                            header::CONTENT_TYPE,
                            HeaderValue::from_static("text/x-glsl; charset=utf-8"),
                        ),
                        (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
                    ],
                    buf,
                )
                    .into_response();
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

pub async fn get_custom_css(State(state): State<Arc<AppState>>) -> Response {
    let path_opt = {
        let guard = state.config.read().await;
        guard.resolved_css_path.clone()
    };

    if let Some(path) = path_opt {
        if let Ok(mut file) = File::open(&path).await {
            let mut buf = String::new();
            if file.read_to_string(&mut buf).await.is_ok() {
                return (
                    [
                        (
                            header::CONTENT_TYPE,
                            HeaderValue::from_static("text/css; charset=utf-8"),
                        ),
                        (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
                    ],
                    buf,
                )
                    .into_response();
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn serve_image(path: PathBuf) -> Response {
    if let Ok(mut file) = File::open(&path).await {
        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).await.is_ok() {
            let mime = if path.extension().is_some_and(|e| e == "png") {
                "image/png"
            } else {
                "image/jpeg"
            };
            return (
                [
                    (header::CONTENT_TYPE, HeaderValue::from_static(mime)),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=31536000, immutable"),
                    ),
                    (
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        HeaderValue::from_static("*"),
                    ),
                ],
                buf,
            )
                .into_response();
        }
    }
    StatusCode::NOT_FOUND.into_response()
}
