use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(album_path: &Path) -> Result<()> {
    let launcher = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(launcher).arg(album_path).spawn()?;
    Ok(())
}
