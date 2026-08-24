use anyhow::Result;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run(album_path: &Path) -> Result<()> {
    let manifest_path = album_path.join("metadata.toml");
    if manifest_path.exists() {
        let launcher = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let mut cmd = Command::new(launcher);
        cmd.arg(manifest_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd.process_group(0);
        cmd.spawn()?;
    }
    Ok(())
}
