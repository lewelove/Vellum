File: rust/src/compile/resolvers/mod.rs
Role: Native Metadata Resolver Index

Description:
This module acts as a switchboard that routes internal metadata keys to their specific native calculation logic. It dictates how the system natively generates specific fields without needing an external Python/JS extension.

Imports:
`pub mod native; pub mod standard;`
- Exposes sub-modules containing complex native calculations and standard retrieval logic.
`use crate::compile::builder::context::{AlbumContext, TrackContext};`
- Brings in the compilation contexts.
`use serde_json::{Value, json};`

Logic:
`pub fn resolve_album_key(key: &str, ctx: &AlbumContext) -> Option<Value>`
- A massive switch statement routing an album key to its function.
- If the builder encounters a key like `date`, this routes it to `native::resolve_date`. It acts as the internal registry of features, returning the computed JSON value. Keys cover string formatting, date generation, custom logic fallbacks, and image analysis (chroma, entropy).

`pub fn resolve_track_key(key: &str, ctx: &TrackContext) -> Option<Value>`
- The equivalent switch statement for track-level keys.
- Maps things like `title`, `artist`, `tracknumber`, and `discnumber` to standard fallback retrievals (e.g., pulling the artist from the album if the track lacks one).
