File: rust/src/server/api/assets.rs
Role: Image and Lyrics Asset Server

Description:
This file acts as a fast delivery service, providing the user interface with specific files like album cover thumbnails, full-resolution artwork, and synchronized lyrics text.

Imports:
`use crate::server::state::AppState;`
- Connects to the active state of the server.
`use axum::extract::{Path, State};`
- Extracts variables from the URL path.
`use axum::http::{HeaderValue, StatusCode, header};`
- Provides standards for HTTP web responses.
`use axum::response::{IntoResponse, Response};`
- Used to formulate the final web response.
`use std::path::PathBuf;`
- For path handling.
`use tokio::fs::File; use tokio::io::AsyncReadExt;`
- Asynchronous capabilities for opening and reading files without blocking the server.

Logic:
`pub async fn get_cover_thumbnail(Path((size, hash)): Path<(String, String)>, State(state): State<Arc<AppState>>) -> Response`
- Serves an optimized thumbnail image.
- Uses the URL to identify the desired size and unique hash of the cover, locates it within the thumbnail cache directory on disk, and securely serves it.

`pub async fn get_album_cover(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Response`
- Serves the original, full-quality cover image directly from the album's folder.
- Retrieves the album from the live memory database using its ID, finds exactly what the cover image is named, constructs the absolute path, and serves the raw file.

`pub async fn get_lyrics(Path((id, path)): Path<(String, String)>, State(state): State<Arc<AppState>>) -> Response`
- Delivers the raw text content of a lyrics file to the queue interface.
- It securely validates that the lyrics file is located inside the library root, reads its contents into a string, and packages it with standard plaintext headers so the browser can easily digest it.

`async fn serve_image(path: PathBuf) -> Response`
- A specialized helper to blast image files to the browser with heavy caching.
- It reads an image from the drive and explicitly injects HTTP headers telling the user's browser to "cache this forever" (max-age=31536000), meaning the server will rarely need to send the same image twice, making scrolling blazing fast.
