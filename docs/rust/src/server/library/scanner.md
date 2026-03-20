File: rust/src/server/library/scanner.rs
Role: In-Memory Library Engine

Description:
This file contains the logic to scan the hard drive, read every compiled metadata lock, and assemble them into a lightning-fast memory database equipped with instant-lookup maps.

Imports:
`use crate::server::library::models::{AlbumView, LockFile};`
- Pulls in the blueprints.
`use std::collections::HashMap;`
- Used to create high-speed indexes.
`use std::path::{Path, PathBuf};`
`use walkdir::WalkDir;`
- For blitzing through folders.

Logic:
`impl Library`
- The container for all library operations.

`pub fn new(...) -> Self`
- Initializes the empty framework of the library, setting up the lists and lookup maps.

`fn normalize_path(path: &str) -> String`
- Cleans up file paths, stripping leading slashes to prevent lookup errors where `/album/path` fails to match `album/path`.

`pub fn scan(&mut self)`
- Bootstraps the entire live database.
- It aggressively searches the root folder for any `metadata.lock.json` file. For every file it finds, it parses the content into the strict data models. It adds the album to a list, inserts it into an `album_map` for instant lookups by ID, and creates a `track_map` index that connects every single individual audio file path to its parent album ID. This index is critical for allowing the audio player to query what album a specific song belongs to instantly.

`pub fn update_album(&mut self, folder_path_str: &str) -> Option<AlbumView>`
- Hot-swaps a single album in memory.
- Instead of rebuilding the entire database, this targets one specific folder, parses its newly updated lock file, completely replaces its old entry in the maps and lists, and updates all track path indexes seamlessly, taking milliseconds.
