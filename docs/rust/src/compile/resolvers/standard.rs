File: rust/src/compile/resolvers/standard.rs
Role: Basic Metadata Utilities

Description:
A collection of small helper functions for retrieving primitive values safely from JSON payloads without excessive error handling.

Imports:
`use serde_json::Value;`
- Used to interact with JSON data.

Logic:
`pub fn get_raw(source: &Value, key: &str, default: &str) -> String`
- Looks up a key in a JSON object. If it exists and is a string, it returns it; otherwise, it returns the provided default.

`pub fn get_raw_with_fallback(source: &Value, album: &Value, key: &str, album_key: &str, default: &str) -> String`
- First attempts to pull a key from the track source data. If it fails, it falls back to a secondary key in the album data. If both fail, it returns a default. Useful for inheriting Album Artist data when a Track Artist isn't specified.

`pub fn format_ms(ms: u64) -> String`
- Takes a raw duration in milliseconds and mathematically converts it into a human-readable string formatted as `MM:SS` or `H:MM:SS`.
