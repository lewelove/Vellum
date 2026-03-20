File: rust/src/manifest/grouper.rs
Role: Track Grouping and Sorting Authority

Description:
When scanning raw audio files, this file is responsible for logically linking individual songs together into complete albums based on shared tags, sorting them correctly, and verifying that the folder containing them is exclusively dedicated to that single album.

Imports:
`use serde_json::Value;`
- Used to read the tags of the tracks.
`use std::collections::{HashMap, HashSet};`
- Used to group tracks into buckets and ensure paths are unique.
`use std::path::{Path, PathBuf};`
- Used to analyze and manipulate folder locations.

Logic:
`pub fn normalize_tag(value: Option<&Value>) -> String`
- Cleans up tags to prevent grouping errors caused by formatting differences.
- It takes a raw data tag, strips away invisible characters, quotes, or trailing spaces, and converts lists into a simple string. This ensures that a track with the artist "Beatles" and another track with " Beatles " are recognized as belonging to the same group.

`fn parse_sort_int(value: Option<&Value>) -> u32`
- Safely converts track and disc numbers into math-friendly integers.
- Because audio tags sometimes store track numbers strangely (like "01/12" instead of just "1"), this extracts the first number and mathematically converts it so tracks can be correctly sorted in order.

`pub fn group_tracks(tracks: Vec<(PathBuf, serde_json::Map<String, Value>)>, keys: &[String]) -> HashMap<Vec<String>, Vec<(PathBuf, serde_json::Map<String, Value>)>>`
- Buckets independent audio files together into unified albums.
- It iterates over every unorganized track, extracts its critical grouping tags (like Album Name and Album Artist), and throws the track into a specific bucket matching those exact tags. Returns a map of separate album groupings.

`pub fn sort_album_tracks(tracks: &mut[(PathBuf, serde_json::Map<String, Value>)])`
- Arranges the tracks of an album into perfect chronological sequence.
- It sorts the list of tracks by evaluating their disc number first, then their track number. If two files accidentally have the exact same numbers, it falls back to sorting them based on their file names (alphabetically) to ensure predictable order.

`pub fn resolve_anchor(tracks: &[(PathBuf, serde_json::Map<String, Value>)], library_root: &Path, supported_exts: &[String]) -> (Option<PathBuf>, bool)`
- Finds the common folder that holds all the tracks and verifies it is safe to use as an album root.
- It analyzes the file paths of every track in the group to find their highest shared parent directory (the anchor). It then checks this anchor directory to ensure no other stray audio files exist inside it that belong to a *different* album. This strict rule ("ecological exclusivity") ensures that one folder equals exactly one album, rejecting the folder if there is a collision.
