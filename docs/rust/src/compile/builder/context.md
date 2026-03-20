File: rust/src/compile/builder/context.rs
Role: Compilation Data Context Definition

Description:
This file defines the primary data structures (contexts) used to carry information throughout the album compilation process. It neatly packages everything needed to build the final metadata object for an album or a track.

Imports:
`use crate::harvest::TrackJson;`
- Imports the structure holding raw physical data extracted from audio files.
`use image::DynamicImage;`
- Used to reference the loaded album cover image in memory.
`use serde_json::Value;`
- Used to hold loosely structured metadata from the user's sidecar files.
`use std::path::Path;`
- Used for holding paths to the library and album roots.

Logic:
`pub struct AlbumContext<'a>`
- Acts as a container for all album-level data needed during compilation.
- This structure holds references to the sidecar metadata (`source`), the compiled track data, the absolute paths to the album and library root, details about the sidecar file's hash and modification time, and all the physical characteristics of the album cover (hash, path, size, mtime, and optionally the loaded image itself). By passing this single context structure around, the compiler can easily access any piece of information without needing a dozen function arguments.

`pub struct TrackContext<'a>`
- Acts as a container for all track-level data needed during compilation.
- Similar to `AlbumContext`, this holds references specifically scoped to a single track. It includes the track's raw harvested data (physics and native tags), the track's explicit metadata from the sidecar (`source`), the album's metadata (in case a track needs to fall back to an album-level property like artist), and its contextual numbers like ordinal track and disc numbers.
