use anyhow::{Context, Result};
use lofty::config::ParseOptions;
use lofty::file::AudioFile;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::TagType;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
pub struct TrackJson {
    pub path: PathBuf,
    pub tags: HashMap<String, String>,
    pub physics: PhysicsData,
}

#[derive(Serialize)]
pub struct PhysicsData {
    pub file_size: u64,
    pub mtime: u64,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub bit_depth: Option<u8>,
    pub channels: u8,
    pub audio_bitrate: u32,
    pub overall_bitrate: u32,
    pub format: String,
}

pub fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

pub fn harvest_file(path: &Path) -> Result<TrackJson> {
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len();
    let mtime = metadata
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let mut f = std::fs::File::open(path).context("Open failed")?;
    let probe = Probe::new(&mut f).guess_file_type().context("Guess failed")?;
    let file_type = probe.file_type();

    let tagged_file = Probe::open(path)?
        .options(ParseOptions::new().read_cover_art(false))
        .read()
        .context("Read failed")?;

    if let Some(lofty::file::FileType::Flac) = file_type {
        if tagged_file.tag(TagType::Id3v2).is_some() {
            log::warn!(
                "ID3v2 tag encountered in FLAC (incompatible with standards): {}",
                path.display()
            );
        }
    }

    let properties = tagged_file.properties();

    let physics = PhysicsData {
        file_size,
        mtime,
        duration_ms: u64::try_from(properties.duration().as_millis()).unwrap_or(u64::MAX),
        sample_rate: properties.sample_rate().unwrap_or(0),
        bit_depth: properties.bit_depth(),
        channels: properties.channels().unwrap_or(0),
        audio_bitrate: properties.audio_bitrate().unwrap_or(0),
        overall_bitrate: properties.overall_bitrate().unwrap_or(0),
        format: format!("{:?}", tagged_file.file_type()),
    };

    let mut tags = HashMap::new();
    let mut concrete_parsed = false;

    let mut file_content = std::fs::File::open(path)?;
    match file_type {
        Some(lofty::file::FileType::Flac) => {
            if let Ok(flac) = lofty::flac::FlacFile::read_from(
                &mut file_content,
                ParseOptions::new().read_cover_art(false),
            ) {
                if let Some(comments) = flac.vorbis_comments() {
                    for (k, v) in comments.items() {
                        let key = sanitize_key(k);
                        let value = v.trim();
                        if !key.is_empty() && !value.is_empty() {
                            tags.entry(key)
                                .and_modify(|e: &mut String| {
                                    if !e.contains(value) {
                                        e.push_str("; ");
                                        e.push_str(value);
                                    }
                                })
                                .or_insert_with(|| value.to_string());
                        }
                    }
                    concrete_parsed = true;
                }
            }
        }
        Some(lofty::file::FileType::Vorbis) => {
            if let Ok(ogg) = lofty::ogg::VorbisFile::read_from(
                &mut file_content,
                ParseOptions::new().read_cover_art(false),
            ) {
                let comments = ogg.vorbis_comments();
                for (k, v) in comments.items() {
                    let key = sanitize_key(k);
                    let value = v.trim();
                    if !key.is_empty() && !value.is_empty() {
                        tags.entry(key)
                            .and_modify(|e: &mut String| {
                                if !e.contains(value) {
                                    e.push_str("; ");
                                    e.push_str(value);
                                }
                            })
                            .or_insert_with(|| value.to_string());
                    }
                }
                concrete_parsed = true;
            }
        }
        Some(lofty::file::FileType::Opus) => {
            if let Ok(opus) = lofty::ogg::OpusFile::read_from(
                &mut file_content,
                ParseOptions::new().read_cover_art(false),
            ) {
                let comments = opus.vorbis_comments();
                for (k, v) in comments.items() {
                    let key = sanitize_key(k);
                    let value = v.trim();
                    if !key.is_empty() && !value.is_empty() {
                        tags.entry(key)
                            .and_modify(|e: &mut String| {
                                if !e.contains(value) {
                                    e.push_str("; ");
                                    e.push_str(value);
                                }
                            })
                            .or_insert_with(|| value.to_string());
                    }
                }
                concrete_parsed = true;
            }
        }
        _ => {}
    }

    if !concrete_parsed {
        if let Some(tag) = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
        {
            let tag_type = tag.tag_type();
            for item in tag.items() {
                let key_raw = item
                    .key()
                    .map_key(tag_type)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| format!("{:?}", item.key()));
                let key = sanitize_key(&key_raw);

                let Some(value) = item.value().text() else {
                    continue;
                };
                let value = value.trim();

                if key.is_empty() || value.is_empty() {
                    continue;
                }

                tags.entry(key)
                    .and_modify(|existing: &mut String| {
                        if !existing.contains(value) {
                            existing.push_str("; ");
                            existing.push_str(value);
                        }
                    })
                    .or_insert_with(|| value.to_string());
            }
        }
    }

    Ok(TrackJson {
        path: path.to_path_buf(),
        tags,
        physics,
    })
}

