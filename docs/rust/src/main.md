File: rust/src/main.rs
Role: CLI Application Entry Point

Description:
This file serves as the main executable wrapper. It defines the Command-Line Interface (CLI), parses user arguments, establishes system-wide logging, and delegates control to the respective backend modules.

Imports:
`use anyhow::{Context, Result};`
`use clap::{Parser, Subcommand};`
- The framework used to cleanly define CLI arguments, flags, and help menus.
`use std::path::PathBuf;`

Logic:
`struct Cli / enum Commands`
- Uses declarative macros to define the exact shape of the CLI interactions (e.g., `vellum server`, `vellum compile --force`). Maps command-line flags to strictly typed internal variables.

`pub fn expand_path(path_str: &str) -> PathBuf`
- A critical and ubiquitous utility function. Since the Rust backend deals heavily with user-provided configuration strings, this safely translates Bash-style tilde paths (like `~/.config/`) into absolute machine paths native to the operating system, preventing constant file-not-found errors.

`async fn main() -> Result<()>`
- The ignition sequence for the entire binary.
- It sets up the `simple_logger` globally, filtering out excessively noisy logs from network dependencies. It parses the commands from the terminal. If threading overrides are requested, it builds the global Rayon thread pool. Finally, a large `match` statement routes the program execution to the desired core module (`harvest::run`, `server::run`, `compile::run`, etc.).
