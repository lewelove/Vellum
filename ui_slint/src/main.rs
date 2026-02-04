use slint::{SharedString, VecModel};
use std::rc::Rc;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
use toml::Value;

slint::include_modules!();

#[derive(Debug, Deserialize)]
struct LockFile {
    album: AlbumInfo,
}

#[derive(Debug, Deserialize)]
struct AlbumInfo {
    #[serde(rename = "ALBUM")]
    album: String,
    #[serde(rename = "ALBUMARTIST")]
    album_artist: String,
    cover_hash: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = MainWindow::new()?;

    let mut library_root = PathBuf::from(".");
    
    if let Ok(config_content) = fs::read_to_string("../config.toml") {
        if let Ok(config_toml) = toml::from_str::<Value>(&config_content) {
            if let Some(root) = config_toml.get("storage")
                .and_then(|s| s.get("library_root"))
                .and_then(|r| r.as_str()) 
            {
                library_root = PathBuf::from(root);
            }
        }
    }

    println!("Scanning library at: {:?}", library_root);

    let mut album_data_list = Vec::new();

    for entry in WalkDir::new(library_root)
        .max_depth(5) 
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) 
    {
        if entry.file_name() == "metadata.lock.json" {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(data) = serde_json::from_str::<LockFile>(&content) {
                    album_data_list.push(AlbumData {
                        title: SharedString::from(data.album.album),
                        artist: SharedString::from(data.album.album_artist),
                        cover_hash: SharedString::from(data.album.cover_hash.unwrap_or_default()),
                    });
                }
            }
        }
    }

    println!("Found {} albums with lock files.", album_data_list.len());

    if album_data_list.is_empty() {
        for i in 1..=20 {
            album_data_list.push(AlbumData {
                title: format!("Fallback Album {}", i).into(),
                artist: "Unknown Artist".into(),
                cover_hash: "".into(),
            });
        }
    }

    let model = Rc::new(VecModel::from(album_data_list));
    ui.set_albums(model.into());

    ui.run()?;
    Ok(())
}
