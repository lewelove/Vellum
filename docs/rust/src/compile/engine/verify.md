File: rust/src/compile/engine/verify.rs
Role: Audio Tag Synchronization Verifier

Description:
This file is responsible for comparing the newly compiled metadata object against the raw physical tags embedded in the audio files. It determines if a write operation is necessary to synchronize the files with the library.

Imports:
`use serde_json::Value;`
- Used to read the compiled and physical metadata structures.
`use std::collections::HashMap;`
- Used to reference the registry configuration.

Logic:
`pub fn calculate_file_tag_subset_match(enriched: &Value, harvest: &[Value], registry: &HashMap<String, Value>) -> bool`
- Determines if the physical tags perfectly reflect the compiled intent.
- It extracts the album block and track array from the compiled object. It first compares the length of compiled tracks against the physical files. Then, for every track, it iterates through core standard tags (like `ALBUM`, `ARTIST`, `TITLE`, `TRACKNUMBER`). It compares the value defined in the compiled object against the physical tag extracted during harvesting. After checking core tags, it iterates through the dynamic registry to check custom user tags that are flagged for synchronization. If any tag mismatches, it immediately returns `false` (meaning the files are "dirty" and need to be synced). If all checked tags perfectly match, it returns `true`.

`fn compare_values(key: &str, compiled: &Value, physical: &str) -> bool`
- A smart comparison function that handles formatting discrepancies between plain strings, lists, and numbers.
- Because audio tags represent data differently (e.g., an array of genres in JSON vs a semicolon-separated string in a FLAC file), this normalizes the compiled value into a standard string format. For track and disc numbers, it implements specific logic to parse strings like "1/12" (Track 1 out of 12) down to the integer `1` for safe mathematical comparison. It then checks if the normalized compiled string matches the physical string.
