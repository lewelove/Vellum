File: rust/src/manifest/compressor.rs
Role: Manifest Data Optimizer

Description:
During initial manifest generation, this logic looks at the tags of every track and determines which tags are universally identical across the album. It "promotes" identical tags to the album level, keeping the sidecar file DRY (Don't Repeat Yourself) and highly readable.

Imports:
`use indexmap::IndexMap;`
- Ordered hashmap to preserve the user's config layout sequence.
`use serde_json::Value;`
`use std::collections::HashSet;`
`use toml::Value as TomlValue;`

Logic:
`pub fn compress(mut raw_tracks: Vec<serde_json::Map<String, Value>>, manifest_layout: Option<&IndexMap<String, TomlValue>>) -> (serde_json::Map<String, Value>, Vec<serde_json::Map<String, Value>>)`
- Evaluates a group of tracks and separates tags into album-pool and track-pool.
- It parses the user's layout configuration to find "forced" keys—keys that the user explicitly wants to keep on the track level regardless of duplication (like `TITLE` or `TRACKNUMBER`). It then takes the tags of the very first track and checks every subsequent track. If a tag's value is absolutely identical across every single track, and the tag is not in the "forced" list, the tag is deleted from all tracks and moved to the `album_pool` map. It returns the extracted album dictionary and the cleaned track array.
