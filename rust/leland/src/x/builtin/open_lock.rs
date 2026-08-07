use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(album_path: &Path) -> Result<()> {
    let lock_path = album_path.join("album.lock.json");
    if lock_path.exists() {
        let launcher = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        Command::new(launcher).arg(lock_path).spawn()?;
    }
    Ok(())
}
