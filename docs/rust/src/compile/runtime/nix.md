File: rust/src/compile/runtime/nix.rs
Role: Nix Environment Integrator

Description:
When extensions require specific external system dependencies (like FFmpeg or ImageMagick), they are defined in a `flake.nix` file. This logic invokes Nix to resolve the environment variables and dependencies, ensuring the kernel process runs in a perfectly reproducible environment.

Imports:
`use anyhow::{Context, Result};`
`use serde_json::Value;`
`use std::collections::HashMap;`
`use std::fs;`
`use std::path::{Path, PathBuf};`
`use std::process::Command;`
`use std::time::SystemTime;`

Logic:
`pub fn get_nix_env(project_root: &Path, explicit_flake: Option<PathBuf>) -> Result<HashMap<String, String>>`
- Resolves and caches the Nix development environment variables.
- It checks if a `flake.nix` exists. If so, it calculates a cache key based on the modification time of the flake file. It checks a hidden cache directory to see if these variables have already been computed for this exact timestamp. If a cache hit occurs, it instantly returns the cached variables, avoiding a slow Nix evaluation. If no cache exists, it shells out to the OS and executes `nix print-dev-env --json`. It parses the massive JSON response containing hundreds of paths and environment variables, extracts them into a standard Rust HashMap, writes them to the cache file for next time, and returns them to be injected into the kernel subprocess.
