File: rust/src/compile/builder/mod.rs
Role: Album Object Builder

Description:
This is the core compilation logic that merges user-defined sidecar metadata with physically harvested audio data. It handles the validation of tracks, invokes physical scans, builds the final JSON structure for the album, and determines if external extensions are needed.

Imports:
`pub mod assets; pub mod context; pub mod scan;`
- Exposes sub-modules for asset handling, context structures, and file scanning.
`use crate::compile::builder::context::{AlbumContext, TrackContext};`
- Imports the context structures to hold data during the build process.
`use crate::compile::resolvers;`
- Imports logic that automatically resolves specific metadata fields.
`use crate::expand_path;`
- Used to resolve user paths.
`use crate::harvest;`
- Imports the module responsible for reading raw audio file data.
`use anyhow::{Result, anyhow};`
- Used for robust error handling and creating custom error messages.
`use serde_json::{Value, json};`
- Used to construct the final compiled JSON object.
`use sha2::Digest;`
- Used for hashing the metadata sidecar file.
`use std::collections::HashSet;`
- Used for detecting duplicate tracks.
`use std::path::Path;`
- Used for interacting with filesystem paths.

Logic:
`pub fn build(...) -> Result<(Value, bool)>`
- Orchestrates the entire compilation process for a single album directory.
- It begins by finding the `metadata.toml` sidecar, reading it, and generating a hash to track changes. It then resolves the album cover information and loads it. It scans the directory for audio files that match supported extensions and checks if the amount of audio files matches the amount of tracks defined in the metadata. After verifying track indices to avoid collisions, it triggers the processing of all tracks. Finally, it builds the `AlbumContext`, resolves all album-level metadata, constructs the final unified JSON object combining the album and track data, and returns it alongside a boolean flag indicating if Python/JS extensions are required.

`fn process_tracks(...) -> Result<(Vec<Value>, Vec<Value>, bool)>`
- Iterates over the physical audio files and merges them with their metadata definitions.
- It first physically "harvests" the tags and physical metrics (bitrate, duration) directly from every audio file. It calculates the total number of discs in the album. Then, it creates a `TrackContext` for each track and builds the track object by combining user metadata, harvested data, and dynamically resolved data. It also monitors if any track requires a dynamically resolved extension tag, returning the compiled tracks, the raw harvested data cache, and the extension requirement flag.

`fn validate_track_indices(entries: &[Value], root: &Path) -> Result<()>`
- Ensures that no two tracks are assigned to the same disc and track number.
- It iterates over the tracks, extracting the `discnumber` and `tracknumber`. It places these pairs into a `HashSet`. If a pair is already in the set, it means the user accidentally numbered two tracks identically, and it throws a strict error to halt compilation.

`fn normalize_keys(v: Value) -> Value`
- Recursively converts all keys in a JSON object to lowercase.
- Because users might type `ALBUM`, `Album`, or `album` in their TOML files, this normalizes everything to lowercase at the very beginning to ensure the compiler can predictably look up keys without worrying about case sensitivity.

`fn construct_track_info(ctx: &TrackContext, total_discs: u32) -> Value`
- Constructs the technical `info` block for a compiled track.
- It extracts and formats non-user metadata, such as absolute/relative library paths, audio format encoding, sample rate, bit depth, and duration. It also attempts to automatically locate a synchronized lyrics file (.lrc or .txt) belonging to the track.

`fn build_track(...) -> (Value, bool)`
- Assembles the final user-facing metadata object for a track.
- It merges the technical `info` block with required core tags (`TITLE`, `ARTIST`, `TRACKNUMBER`, `DISCNUMBER`). Then, it iterates over the global `compiler_registry` configuration to see what extra tags the system expects. If the tag is standard, it resolves it. If the tag belongs to an external extension, it flags that an extension pass is required and places a `null` placeholder.

`fn construct_album_info(ctx: &AlbumContext) -> Value`
- Constructs the technical `info` block for the compiled album.
- It sums up the duration of all individual tracks to get the total album runtime. It calculates total tracks and discs, and attaches the paths, hashes, and modification times of both the sidecar metadata file and the album cover image.

`fn build_album(...) -> (Value, bool)`
- Assembles the final user-facing metadata object for the album.
- Similar to `build_track`, it merges the album's `info` block with core tags (`ALBUM`, `ALBUMARTIST`, `DATE`, `GENRE`, etc.). It evaluates the `compiler_registry` for album-level tags, resolving native ones and flagging external extensions if necessary. It outputs the finalized JSON map.
