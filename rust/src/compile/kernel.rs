use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};
use toml::Value;

pub fn spawn(config: &Value, project_root: &Path, env: &HashMap<String, String>) -> Result<Child> {
    let (program, args) = resolve_command(config);

    Command::new(&program)
        .args(&args)
        .envs(env)
        .current_dir(project_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("Failed to spawn kernel: {program} {args:?}"))
}

fn resolve_command(config: &Value) -> (String, Vec<String>) {
    let kernel_cmd_config = config
        .get("extensions")
        .and_then(|c| c.get("kernel_command"))
        .and_then(Value::as_str);

    kernel_cmd_config.map_or_else(
        || {
            let kernel_script = "extensions/javascript/compiler_kernel.js";
            (
                "bun".to_string(),
                vec![
                    "run".to_string(),
                    kernel_script.to_string(),
                ],
            )
        },
        |cmd| {
            let mut parts = cmd.split_whitespace();
            let p = parts.next().unwrap_or("bun").to_string();
            let a = parts.map(ToString::to_string).collect();
            (p, a)
        },
    )
}
