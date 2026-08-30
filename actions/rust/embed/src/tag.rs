use crate::mapping::canonical_key;
use crate::models::{CoverDeleteMode, CoverStatus, DiskCover, TagDeleteMode, TrackTask};
use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use lofty::TextEncoding;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::id3::v2::{
    Frame, FrameId, Id3v2Tag, TextInformationFrame, UniqueFileIdentifierFrame,
    UnsynchronizedTextFrame,
};
use lofty::picture::{Picture, PictureType};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::items::Timestamp;
use lofty::tag::{ItemKey, ItemValue, Tag, TagType};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;
use xxhash_rust::xxh64::xxh64;

const HASH_SEED: u64 = 0;

#[must_use]
pub fn get_hash(data: &[u8]) -> String {
    let h = xxh64(data, HASH_SEED).to_be_bytes();
    URL_SAFE_NO_PAD.encode(h)
}

pub fn read_file_tag(path: &Path) -> Result<(Tag, Option<String>)> {
    let tagged_file = Probe::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?
        .options(ParseOptions::new().read_cover_art(true))
        .guess_file_type()
        .with_context(|| format!("Failed to guess file type: {}", path.display()))?
        .read()
        .with_context(|| format!("Failed to read tags: {}", path.display()))?;

    let primary_type = tagged_file.primary_tag_type();
    let tag = tagged_file.primary_tag().cloned().unwrap_or_else(|| {
        let mut new_tag = Tag::new(primary_type);
        if let Some(first) = tagged_file.first_tag() {
            for item in first.items() {
                new_tag.insert(item.clone());
            }
            for pic in first.pictures() {
                new_tag.push_picture(pic.clone());
            }
        }
        new_tag
    });

    let cover_hash = tag
        .pictures()
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| tag.pictures().first())
        .map(|p| get_hash(p.data()));

    Ok((tag, cover_hash))
}

pub fn extract_tag_map(tag: &Tag) -> HashMap<ItemKey, String> {
    let mut map = HashMap::new();

    for item in tag.items() {
        let key = canonical_key(item.key());
        let value = match item.value() {
            ItemValue::Text(text) | ItemValue::Locator(text) => text.trim(),
            ItemValue::Binary(_) => {
                continue;
            }
        };

        if !value.is_empty() {
            map.entry(key)
                .and_modify(|existing: &mut String| {
                    existing.push_str("; ");
                    existing.push_str(value);
                })
                .or_insert_with(|| value.to_string());
        }
    }

    map
}

pub fn read_disk_cover(cover_path: Option<&Path>) -> Result<Option<DiskCover>> {
    let Some(path) = cover_path else {
        return Ok(None);
    };
    if !path.is_file() {
        anyhow::bail!("Cover image not found: {}", path.display());
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("Failed to read cover image: {}", path.display()))?;
    let hash = get_hash(&bytes);
    Ok(Some(DiskCover {
        path: path.to_path_buf(),
        hash,
    }))
}

pub fn resolve_cover_status(
    tasks: &[TrackTask],
    disk_cover: Option<&DiskCover>,
) -> Result<CoverStatus> {
    let Some(disk) = disk_cover else {
        return Ok(CoverStatus::Preserve);
    };

    for track in tasks {
        let (tag, embedded_hash) = read_file_tag(&track.path)?;
        if embedded_hash.as_deref() != Some(&disk.hash) {
            return Ok(CoverStatus::Update);
        }
        let has_front = tag
            .pictures()
            .iter()
            .any(|p| p.pic_type() == PictureType::CoverFront);
        if !has_front {
            return Ok(CoverStatus::Update);
        }
    }

    Ok(CoverStatus::Preserve)
}

fn apply_item_to_id3v2(id3v2: &mut Id3v2Tag, key: ItemKey, val: &str) {
    match key {
        ItemKey::TrackNumber => {
            id3v2.set_track(val.parse().unwrap_or(0));
        }
        ItemKey::TrackTotal => {
            id3v2.set_track_total(val.parse().unwrap_or(0));
        }
        ItemKey::DiscNumber => {
            id3v2.set_disk(val.parse().unwrap_or(0));
        }
        ItemKey::DiscTotal => {
            id3v2.set_disk_total(val.parse().unwrap_or(0));
        }
        ItemKey::Comment => {
            id3v2.set_comment(val.to_string());
        }
        ItemKey::Lyrics | ItemKey::UnsyncLyrics => {
            let frame = Frame::UnsynchronizedText(UnsynchronizedTextFrame::new(
                TextEncoding::UTF8,
                *b"eng",
                "",
                val.to_string(),
            ));
            id3v2.insert(frame);
        }
        ItemKey::MusicBrainzRecordingId => {
            let frame = Frame::UniqueFileIdentifier(UniqueFileIdentifierFrame::new(
                "http://musicbrainz.org",
                val.as_bytes().to_vec(),
            ));
            id3v2.insert(frame);
        }
        ItemKey::RecordingDate | ItemKey::Year => {
            if let Ok(ts) = val.parse::<Timestamp>() {
                id3v2.set_date(ts);
            } else {
                id3v2.insert(Frame::Text(TextInformationFrame::new(
                    FrameId::Valid(Cow::Borrowed("TDRC")),
                    TextEncoding::UTF8,
                    val.to_string(),
                )));
            }
        }
        ItemKey::Bpm | ItemKey::IntegerBpm => {
            let bpm_str = val.parse::<u16>().map_or_else(
                |_| {
                    val.parse::<f32>().map_or_else(
                        |_| "0".to_string(),
                        |f| {
                            if f.is_finite() && f > 0.0 {
                                format!("{:.0}", f.round())
                            } else {
                                "0".to_string()
                            }
                        },
                    )
                },
                |n| n.to_string(),
            );
            let frame_id = FrameId::Valid(Cow::Borrowed("TBPM"));
            id3v2.insert(Frame::Text(TextInformationFrame::new(
                frame_id,
                TextEncoding::UTF8,
                bpm_str,
            )));
        }
        other => {
            if let Some(mapped) = other.map_key(TagType::Id3v2) {
                if mapped.len() == 4 {
                    let frame_id = FrameId::Valid(Cow::Borrowed(mapped));
                    id3v2.insert(Frame::Text(TextInformationFrame::new(
                        frame_id,
                        TextEncoding::UTF8,
                        val.to_string(),
                    )));
                } else {
                    id3v2.insert_user_text(mapped.to_string(), val.to_string());
                }
            }
        }
    }
}

fn apply_id3v2_tags(
    path: &Path,
    target_tags: &HashMap<ItemKey, String>,
    new_picture: Option<&Picture>,
    delete_tags: TagDeleteMode,
    delete_covers: CoverDeleteMode,
) -> Result<()> {
    let (existing_tag, _) = read_file_tag(path)?;
    let mut id3v2 = Id3v2Tag::new();

    if delete_tags == TagDeleteMode::PreserveOther {
        let existing_id3v2: Id3v2Tag = existing_tag.clone().into();
        for frame in existing_id3v2 {
            if !matches!(frame, Frame::Picture(_)) {
                id3v2.insert(frame);
            }
        }
    }

    for (&k, v) in target_tags {
        apply_item_to_id3v2(&mut id3v2, k, v);
    }

    if let Some(pic) = new_picture {
        if delete_covers == CoverDeleteMode::PreserveOther {
            for existing_pic in existing_tag.pictures() {
                if existing_pic.pic_type() != PictureType::CoverFront {
                    id3v2.insert_picture(existing_pic.clone());
                }
            }
        }
        id3v2.insert_picture(pic.clone());
    } else if delete_covers == CoverDeleteMode::DeleteOther {
        for existing_pic in existing_tag.pictures() {
            if existing_pic.pic_type() == PictureType::CoverFront {
                id3v2.insert_picture(existing_pic.clone());
            }
        }
    } else {
        for existing_pic in existing_tag.pictures() {
            id3v2.insert_picture(existing_pic.clone());
        }
    }

    id3v2.save_to_path(path, WriteOptions::default())?;
    Ok(())
}

fn apply_generic_tags(
    path: &Path,
    mut tag: Tag,
    target_tags: &HashMap<ItemKey, String>,
    new_picture: Option<&Picture>,
    delete_tags: TagDeleteMode,
    delete_covers: CoverDeleteMode,
) -> Result<()> {
    if delete_tags == TagDeleteMode::DeleteOther {
        let target_keys: HashSet<ItemKey> = target_tags.keys().copied().collect();
        tag.retain(|item| {
            let key = canonical_key(item.key());
            key == ItemKey::EncoderSoftware
                || key == ItemKey::EncodedBy
                || target_keys.contains(&key)
        });
    }

    for (k, v) in target_tags {
        tag.insert_text(*k, v.clone());
    }

    if let Some(pic) = new_picture {
        if delete_covers == CoverDeleteMode::DeleteOther {
            while !tag.pictures().is_empty() {
                tag.remove_picture(0);
            }
        } else {
            tag.remove_picture_type(PictureType::CoverFront);
        }
        tag.push_picture(pic.clone());
    } else if delete_covers == CoverDeleteMode::DeleteOther {
        let mut i = 0;
        while i < tag.pictures().len() {
            if tag.pictures()[i].pic_type() == PictureType::CoverFront {
                i += 1;
            } else {
                tag.remove_picture(i);
            }
        }
    }

    tag.save_to_path(path, WriteOptions::default())?;
    Ok(())
}

fn apply_tags_and_cover(
    path: &Path,
    tag: Tag,
    target_tags: &HashMap<ItemKey, String>,
    new_picture: Option<&Picture>,
    delete_tags: TagDeleteMode,
    delete_covers: CoverDeleteMode,
) -> Result<()> {
    if tag.tag_type() == TagType::Id3v2 {
        apply_id3v2_tags(path, target_tags, new_picture, delete_tags, delete_covers)
    } else {
        apply_generic_tags(
            path,
            tag,
            target_tags,
            new_picture,
            delete_tags,
            delete_covers,
        )
    }
}

pub fn write_tasks(
    tasks: &[TrackTask],
    cover_status: CoverStatus,
    disk_cover: Option<&DiskCover>,
    delete_tags: TagDeleteMode,
    delete_covers: CoverDeleteMode,
) -> Result<()> {
    let new_picture = if cover_status == CoverStatus::Update
        && let Some(cover) = disk_cover
    {
        let mut pic = Picture::from_reader(&mut File::open(&cover.path)?)?;
        pic.set_pic_type(PictureType::CoverFront);
        Some(pic)
    } else {
        None
    };

    for task in tasks {
        let (tag, _) = read_file_tag(&task.path)?;
        apply_tags_and_cover(
            &task.path,
            tag,
            &task.target_tags,
            new_picture.as_ref(),
            delete_tags,
            delete_covers,
        )?;
    }

    Ok(())
}
