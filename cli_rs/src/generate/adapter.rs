use anyhow::{Context, Result};
use lofty::{ItemKey, MimeType, PictureType, Probe, TaggedFileExt};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct TrackData {
    pub path: PathBuf,
    pub tags: HashMap<String, String>,
}

impl TrackData {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.tags.get(key)
    }
}

pub fn read_track_tags(path: &Path) -> Result<TrackData> {
    let tagged_file = Probe::open(path)
        .context("Failed to open file")?
        .read()
        .context("Failed to read file tags")?;

    let tag = match tagged_file.primary_tag() {
        Some(primary) => primary,
        None => match tagged_file.first_tag() {
            Some(first) => first,
            None => return Ok(TrackData { 
                path: path.to_path_buf(), 
                tags: HashMap::new() 
            }),
        },
    };

    let tag_type = tag.tag_type();
    let mut map = HashMap::new();

    for item in tag.items() {
        let key_opt = item.key().map_key(tag_type, false).map(|s| s.to_string());

        let key_string = match key_opt {
            Some(s) => Some(s),
            None => match item.key() {
                ItemKey::Unknown(s) => Some(s.clone()),
                _ => None, 
            }
        };

        let key = match key_string {
            Some(k) => k.to_uppercase(),
            None => continue,
        };

        let value = match item.value().text() {
            Some(v) => v.trim().to_string(),
            None => continue,
        };
        
        if key.is_empty() || value.is_empty() {
            continue;
        }

        map.entry(key)
            .and_modify(|v: &mut String| {
                v.push_str("; ");
                v.push_str(&value);
            })
            .or_insert(value);
    }
    
    map.insert("track_path_absolute".to_string(), path.to_string_lossy().to_string());

    Ok(TrackData {
        path: path.to_path_buf(),
        tags: map,
    })
}

pub fn extract_cover(path: &Path, destination_dir: &Path) -> Result<()> {
    let tagged_file = Probe::open(path)
        .context("Failed to open file")?
        .read()
        .context("Failed to read tags")?;

    let tag = match tagged_file.primary_tag() {
        Some(t) => t,
        None => match tagged_file.first_tag() {
            Some(t) => t,
            None => return Ok(()),
        },
    };

    let pictures = tag.pictures();
    if pictures.is_empty() {
        return Ok(());
    }

    let pic = pictures.iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| pictures.iter().next());

    if let Some(pic) = pic {
        let ext = match pic.mime_type() {
            MimeType::Png => "png",
            MimeType::Jpeg => "jpg",
            _ => "jpg",
        };

        let filename = format!("cover.{}", ext);
        let dest = destination_dir.join(filename);

        fs::write(&dest, pic.data())
            .with_context(|| format!("Failed to write cover to {:?}", dest))?;
    }

    Ok(())
}
