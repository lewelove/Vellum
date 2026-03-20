File: rust/src/compile/runtime/kernel.rs
Role: Extension Subprocess Spawner

Description:
This file is responsible for booting up the background kernel process (usually a Node.js or Python runtime) that will execute user-defined custom metadata extensions.

Imports:
`use anyhow::{Context, Result};`
- For robust error wrapping.
`use std::collections::HashMap;`
- To pass environment variables.
`use std::path::Path;`
- To set the execution directory.
`use std::process::Stdio;`
- To configure standard input/output pipes.
`use tokio::process::{Child, Command};`
- To asynchronously spawn and manage the OS process.
`use toml::Value;`
- To read the command configuration.

Logic:
`pub fn spawn(config: &Value, project_root: &Path, env: &HashMap<String, String>) -> Result<Child>`
- Spawns the kernel process.
- It parses the configuration to determine the exact execution command. It spawns the process using `tokio::process::Command`, explicitly binding its environment variables, setting the working directory to the project root, and capturing its `stdin` and `stdout` through pipes so the rust engine can stream data to and from it.

`fn resolve_command(config: &Value) -> (String, Vec<String>)`
- Helper logic that looks at the `kernel_command` string in the config (e.g. `bun run script.js`) and splits it into the base program (`bun`) and its arguments array (`["run", "script.js"]`) so the OS can execute it correctly.
