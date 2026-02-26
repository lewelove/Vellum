use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};
use toml::Value;

pub fn spawn(config: &Value, project_root: &Path, env: &HashMap<String, String>) -> Result<Child> {
    let (prog, args) = resolve_command(config);
    Command::new(&prog)
        .args(&args)
        .envs(env)
        .current_dir(project_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("Failed to spawn {prog}"))
}

fn resolve_command(config: &Value) -> (String, Vec<String>) {
    let cmd = config
        .get("extensions")
        .and_then(|c| c.get("kernel_command"))
        .and_then(Value::as_str);
    cmd.map_or_else(
        || {
            (
                "bun".to_string(),
                vec![
                    "run".to_string(),
                    "extensions/javascript/compiler_kernel.js".to_string(),
                ],
            )
        },
        |c| {
            let mut p = c.split_whitespace();
            (
                p.next().unwrap_or("bun").to_string(),
                p.map(ToString::to_string).collect(),
            )
        },
    )
}
