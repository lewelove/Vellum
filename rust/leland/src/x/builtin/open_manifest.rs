use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(album_path: &Path) -> Result<()> {
    let manifest_path = album_path.join("metadata.toml");
    if manifest_path.exists() {
        let launcher = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        Command::new(launcher).arg(manifest_path).spawn()?;
    }
    Ok(())
}
