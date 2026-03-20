File: rust/src/manifest/engine.rs
Role: TOML Formatting Engine

Description:
This file is responsible for translating the internal metadata structures of an album into clean, human-readable TOML formatting so they can be written to the `metadata.toml` file on the user's hard drive.

Imports:
`use indexmap::IndexMap;`
- Used to keep the configuration items in a strict order.
`use serde_json::Value;`
- Used to read raw JSON data.
`use std::collections::HashSet;`
- Used for fast tracking of which data tags have already been formatted.

Logic:
`fn format_toml_value(val: &Value) -> String`
- Converts basic data into proper text strings.
- This helper checks what type of data is being formatted (a plain string, a number, a true/false boolean, or a list of items). It applies the correct text formatting rules (like wrapping strings in quotes or lists in brackets) so the final text document is structurally sound.

`pub fn render_toml_block(pool: &serde_json::Map<String, Value>, layout: Option<&IndexMap<String, toml::Value>>, level: &str) -> Vec<String>`
- Arranges data tags according to user preferences and outputs lines of text.
- It looks at a pool of metadata tags (like Title, Artist, Date) and checks the user's layout configuration to see the exact order the user wants these tags to appear in. It iterates through the configuration, formats the tag if it exists in the data pool, and occasionally inserts blank lines to group tags visually. Finally, it gathers any leftover "unknown" tags that weren't in the layout configuration, sorts them alphabetically, and tacks them onto the bottom of the document so no data is ever lost.
