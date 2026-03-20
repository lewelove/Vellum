File: rust/src/config.rs
Role: Master Configuration Loader

Description:
This file defines the entire configuration schema for the Vellum application and implements the complex logic needed to locate, recursively load, and seamlessly merge modular TOML configuration files.

Imports:
`use indexmap::IndexMap;`
- Used to preserve the strict top-to-bottom order of user configuration maps.
`use serde::{Deserialize, Serialize};`
- Data mapping traits.
`use toml::Value;`
- Raw TOML data representations.

Logic:
`pub struct AppConfig` (and nested structs)
- Defines the exact shape of the configuration system, breaking elements down into distinct blocks (Storage, Theme, Extensions, Layout).

`pub fn load() -> Result<(Self, Value, PathBuf)>`
- The global entry point. It resolves where the primary configuration file lives, executes the recursive loading logic to fetch any chained imports, dynamically validates the entire merged schema into the `AppConfig` struct, and returns the usable object.

`fn resolve_config_path() -> Option<PathBuf>`
- A fallback chain for locating settings. It checks for a custom environment variable, then looks in the Linux standard `~/.config/vellum/`, and finally traverses upwards through the current working directory to seamlessly support local development environments.

`fn load_recursive(path: &Path, visited: &mut std::collections::HashSet<PathBuf>) -> Result<Value>`
- An advanced file loader supporting modular configuration architecture.
- It reads a config file and checks for an `import = [...]` array. If found, it navigates to the imported files, reads them, and merges them together. It maintains a `visited` registry to strictly prevent infinite circular loop crashes if two files accidentally import each other.

`fn deep_merge(base: Value, overlay: Value) -> Value`
- A recursive tool for blending TOML tables together. It ensures that local user-specific overrides flawlessly replace generalized baseline settings without destroying surrounding data.
