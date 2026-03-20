File: rust/src/compile/mod.rs
Role: Compilation Director

Description:
This is the entry point for the entire compilation system. It gathers user commands, validates flags, initializes the environments, and invokes the stream engine.

Imports:
`pub mod builder; pub mod engine; pub mod resolvers; pub mod runtime;`
- Exposes all compilation submodules.
`use crate::config::AppConfig;`
- To load application settings.
`use anyhow::{Context, Result};`
- Error handling.
`use serde_json::{Value, json};`
- To manipulate configuration data.
`use std::path::PathBuf;`
`use std::sync::Arc;`
`use tokio::sync::mpsc;`

Logic:
`pub enum CompileMode` & `pub enum ExportTarget` & `pub struct CompileFlags` & `pub struct CompileOptions`
- Structures used to cleanly pack the dozen different flags and states the compiler can operate in (e.g., standard file output vs stdout intermediary debugging).

`pub async fn run(mut options: CompileOptions) -> Result<()>`
- The top-level function called from the CLI.
- It loads the config, determines which albums to scan, and checks if the compiler is operating in "Intermediary" mode (where it just builds the native JSON and dumps it for inspection without running extensions). If running in Standard mode, it evaluates whether any external extensions are actually configured to run. If extensions are needed, it triggers the Nix environment resolution (`runtime::nix::get_nix_env`), spawns the external kernel process, and then hands control over to the `engine::stream::run` orchestrator to begin piping data. If no extensions are needed, it triggers the stream orchestrator purely with native builder threads.
