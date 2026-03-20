File: rust/src/compile/builder/assets.rs
Role: Album Cover Asset Manager

Description:
This file manages the discovery, hashing, and processing of album cover images. It identifies standard cover files in a directory, extracts their basic metadata to detect changes, and handles the generation of optimized thumbnail images for the application to cache and use.

Imports:
`use crate::expand_path;`
- Expands user paths like `~` to absolute filesystem paths.
`use base64::Engine;`
- Provides base64 encoding functionality.
`use base64::engine::general_purpose::URL_SAFE_NO_PAD;`
- Specifies a URL-safe, unpadded base64 encoding format.
`use image::{DynamicImage, GenericImageView};`
- Provides image manipulation and representation types.
`use serde_json::Value;`
- Allows interaction with generic JSON data.
`use sha2::{Digest, Sha256};`
- Provides hashing capabilities for generating unique file signatures.
`use std::path::Path;`
- Used for interacting with filesystem paths.
`use std::time::SystemTime;`
- Used for working with file modification times.

Logic:
`pub fn resolve_cover_info(root: &Path) -> (Option<String>, String, u64, u64)`
- Discovers an album cover in the folder and generates a unique hash based on its state.
- The logic iterates through a list of common cover filenames (`cover.jpg`, `cover.png`, `folder.jpg`, `front.jpg`). As soon as it finds one that exists, it retrieves its modification time (mtime) and file size. It hashes these two values together using SHA-256 and encodes the result as a safe Base64 string. This ensures that any modification or replacement of the cover file will yield a new hash without needing to hash the entire image content, making the process highly efficient. It returns the filename, the hash, the mtime, and the size.

`pub fn load_or_create_thumbnail(config: &Value, album_root: &Path, cover_path: Option<&str>, cover_hash: &str) -> Option<DynamicImage>`
- Ensures a processed thumbnail exists in the cache, creating one if necessary.
- The function reads the user's configuration to find the thumbnail cache directory and the desired thumbnail size. It constructs the path where the cached thumbnail should live based on the unique `cover_hash`. If a thumbnail already exists at this location, it simply loads and returns it. If not, it opens the original cover image, crops it perfectly to a square based on the shortest dimension (to avoid stretching), scales it down to the target size using high-quality Lanczos3 filtering, saves the result to the cache, and returns the image.
