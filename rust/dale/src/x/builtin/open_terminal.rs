use anyhow::Result;
use std::env;
use std::path::Path;
use std::process::Command;

const COMMON_TERMINALS: &[&str] = &[
    "ghostty",
    "kitty",
    "foot",
    "alacritty",
    "wezterm",
    "st",
    "gnome-terminal",
    "konsole",
    "xterm",
];

pub fn run(album_path: &Path, config: &serde_json::Value) -> Result<()> {
    let term_bin = config
        .get("terminal")
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .or_else(|| env::var("TERMINAL").ok())
        .or_else(|| {
            COMMON_TERMINALS
                .iter()
                .find(|&&t| which_exists(t))
                .map(|&t| t.to_string())
        });

    if let Some(term) = term_bin {
        let mut cmd = Command::new(term);
        cmd.current_dir(album_path);

        if let Some(inner_cmd) = config.get("cmd").and_then(|v| v.as_str()) {
            cmd.arg("-e").arg("sh").arg("-c").arg(inner_cmd);
        }

        cmd.spawn()?;
    }
    Ok(())
}

fn which_exists(bin: &str) -> bool {
    env::var_os("PATH")
        .and_then(|paths| {
            env::split_paths(&paths).find_map(|p| {
                let full = p.join(bin);
                if full.is_file() { Some(full) } else { None }
            })
        })
        .is_some()
}
