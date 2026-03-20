File: rust/src/compile/engine/stream.rs
Role: Compilation Execution Pipeline

Description:
This file manages the high-level execution flow of compiling thousands of albums. It parallelizes the building of albums, streams the ones requiring extensions to a background Python/JS kernel process via standard I/O, and catches the results to finalize them.

Imports:
`use crate::compile::{ExportTarget, builder, engine::verify};`
- Brings in the builder logic and verification logic.
`use anyhow::{Context, Result};`
- Standard error handling.
`use rayon::prelude::*;`
- Provides parallel iterator traits for multi-threading.
`use serde_json::{Value, json};`
- JSON manipulation.
`use std::collections::HashMap;`
- Used for hash maps.
`use std::path::{Path, PathBuf};`
- Path handling.
`use std::sync::Arc;`
- Thread-safe reference counting to share data across tasks.
`use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};`
- Asynchronous I/O for talking to the background extension kernel.
`use tokio::process::Child;`
- Representation of the spawned kernel process.
`use tokio::sync::mpsc;`
- Asynchronous message passing channels to route data between threads.

Logic:
`pub struct StreamContext`
- A configuration object holding all parameters required for the streaming pipeline, including the list of albums, configuration references, thread limits, and targets.

`pub async fn run(child: Option<Child>, ctx: StreamContext) -> Result<()>`
- The main orchestrator for the compilation stream.
- It sets up two communication channels: one for albums that require the external extension kernel (`ktx`) and one for albums that are fully resolved natively (`dtx`). It spawns a background thread pool (`spawn_builders`) that parallelizes the initial parsing of all albums. It then spawns a task to directly finalize natively built albums. If an external kernel process was provided, it sets up a bridge (`run_bridge`) to pipe the incomplete albums to the kernel and read the enriched results back. Finally, it awaits for all tasks to complete.

`fn spawn_builders(ctx: &StreamContext, ktx: mpsc::Sender<String>, dtx: mpsc::Sender<Value>) -> tokio::task::JoinHandle<()>`
- Spawns a dedicated OS thread pool using Rayon to build albums in parallel.
- For each album path, it calls `builder::build`. If the result indicates that an extension is required, the JSON is serialized into a string and sent to the `ktx` channel (to be forwarded to Python/JS). If no extension is required, the raw JSON object is sent to the `dtx` channel to be finalized immediately. This completely isolates CPU-heavy disk/parsing work from the async network/IO threads.

`async fn run_bridge(child: &mut Child, mut krx: mpsc::Receiver<String>, ...) -> Result<()>`
- Handles bidirectional communication with the external kernel process.
- It spawns two async tasks: a `sender` that constantly pulls JSON strings from the `krx` channel and writes them into the standard input (stdin) of the kernel process, and a `receiver` that constantly reads lines from the standard output (stdout) of the kernel process. When the kernel finishes enriching an album and spits it back out, the receiver catches it, parses it back into JSON, and schedules it for finalization.

`fn strip_empty_values(value: &mut Value)`
- Recursively traverses a JSON object and removes any keys that have empty strings or `null` values.
- This ensures that the final compiled `metadata.lock.json` file is kept clean and minimal, without tons of undefined or blank metadata keys taking up space.

`fn finalize(mut v: Value, target: ExportTarget, ...) -> Result<()>`
- The final step for every compiled album object.
- It extracts the `ctx` block (which contains temporary runtime data like raw harvested metadata). It passes this temporary data alongside the compiled data to `verify::calculate_file_tag_subset_match` to check if the audio files on disk actually match what was just compiled. The result is attached as a boolean flag `file_tag_subset_match`. The object is stripped of empty values. Finally, depending on the target, it either prints the JSON to the console or writes it to `metadata.lock.json` in the album's folder, sending a notification trigger to update the server's live library state.
