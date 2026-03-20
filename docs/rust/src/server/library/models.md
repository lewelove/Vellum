File: rust/src/server/library/models.rs
Role: Library Data Models

Description:
This file defines the exact blueprints for what an Album and a Track look like in the server's memory. It dictates how the raw data from `metadata.lock.json` is perfectly translated into safe Rust objects.

Imports:
`use serde::{Deserialize, Deserializer, Serialize};`
- Tools to translate raw text into structured objects and vice-versa.
`use std::collections::HashMap;`

Logic:
`pub struct TrackInfo / TrackLock / AlbumInfo / AlbumLock`
- Strict blueprints representing all the expected data fields (like duration, file sizes, titles, and custom tags). Using these blueprints ensures that the server can never accidentally misread or drop critical data from the compilation locks.

`fn deserialize_vec_or_string(...) -> Result<Vec<String>, D::Error>`
- A custom logic block to handle unpredictable user formatting.
- Because a tag like `GENRE` might be saved as a clean list `["Rock", "Pop"]` or a messy string `"Rock; Pop"`, this intercepts the parsing process, safely converts either format into a clean list of strings, and prevents the server from crashing due to formatting errors.

`pub struct LockFile / AlbumView`
- The top-level wrapper objects representing the complete physical package of an album, bundling its info block and track array together.
