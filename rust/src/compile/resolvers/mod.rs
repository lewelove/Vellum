pub mod native;
pub mod standard;

use crate::compile::builder::context::{AlbumContext, TrackContext};
use serde_json::{Value, json};

pub fn resolve_top_level_album_key(key: &str, ctx: &AlbumContext) -> Value {
    match key {
        "ALBUM" => standard::resolve_generic_string(ctx.source, "album", "", "Unknown Album"),
        "ALBUMARTIST" => standard::resolve_generic_string(ctx.source, "albumartist", "artistartist", "Unknown Artist"),
        "DATE" => standard::resolve_generic_string(ctx.source, "date", "year,originalyear", "0000"),
        "GENRE" => {
            let mut list = standard::resolve_generic_list(ctx.source, "genre", "");
            if let Value::Array(ref arr) = list {
                if arr.is_empty() {
                    list = json!(["Unknown"]);
                }
            }
            list
        },
        "COMMENT" => json!(native::resolve_comment(ctx, "")),
        "ORIGINAL_YYYY_MM" => json!(native::resolve_yyyy_mm(ctx, "original_yyyy_mm", "")),
        "RELEASE_YYYY_MM" => json!(native::resolve_yyyy_mm(ctx, "release_yyyy_mm", "")),
        _ => Value::Null,
    }
}

pub fn resolve_top_level_track_key(key: &str, ctx: &TrackContext) -> Value {
    match key {
        "TITLE" => standard::resolve_generic_string(ctx.source, "title", "", "Untitled"),
        "ARTIST" => standard::resolve_generic_string_fallback(
            ctx.source, ctx.album_source, "artist", "albumartist", "Unknown Artist"
        ),
        _ => Value::Null,
    }
}

pub fn resolve_album_key(key: &str, meta: &Value, ctx: &AlbumContext) -> Option<Value> {
    let class = meta.get("class").and_then(Value::as_str).unwrap_or("generic");
    let type_ = meta.get("type").and_then(Value::as_str).unwrap_or("string");
    let args = meta.get("args").and_then(Value::as_str).unwrap_or("");

    if class == "function" {
        let res = match key {
            "cover_chroma" => native::resolve_cover_chroma(ctx, args),
            "cover_entropy" => native::resolve_cover_entropy(ctx, args),
            "cover_palette" => native::resolve_cover_palette(ctx, args),
            "original_yyyy_mm" => Some(json!(native::resolve_yyyy_mm(ctx, "original_yyyy_mm", args))),
            "release_yyyy_mm" => Some(json!(native::resolve_yyyy_mm(ctx, "release_yyyy_mm", args))),
            "comment" => Some(json!(native::resolve_comment(ctx, args))),
            "unix_added" => Some(json!(native::resolve_album_info_unix_added(ctx, args))),
            _ => {
                log::warn!("Native function for key '{}' not found, falling back to generic.", key);
                None
            }
        };
        if res.is_some() {
            return res;
        }
    }

    match type_ {
        "list" => Some(standard::resolve_generic_list(ctx.source, key, args)),
        "integer" => Some(standard::resolve_generic_integer(ctx.source, key, args)),
        "float" => Some(standard::resolve_generic_float(ctx.source, key, args)),
        "bool" => Some(standard::resolve_generic_bool(ctx.source, key, args)),
        "string" | _ => Some(standard::resolve_generic_string(ctx.source, key, args, "")),
    }
}

pub fn resolve_track_key(key: &str, meta: &Value, ctx: &TrackContext) -> Option<Value> {
    let class = meta.get("class").and_then(Value::as_str).unwrap_or("generic");
    let type_ = meta.get("type").and_then(Value::as_str).unwrap_or("string");
    let args = meta.get("args").and_then(Value::as_str).unwrap_or("");

    if class == "function" {
        let res = match key {
            _ => {
                log::warn!("Native function for track key '{}' not found, falling back to generic.", key);
                None
            }
        };
        if res.is_some() {
            return res;
        }
    }

    match type_ {
        "list" => Some(standard::resolve_generic_list(ctx.source, key, args)),
        "integer" => Some(standard::resolve_generic_integer(ctx.source, key, args)),
        "float" => Some(standard::resolve_generic_float(ctx.source, key, args)),
        "bool" => Some(standard::resolve_generic_bool(ctx.source, key, args)),
        "string" | _ => Some(standard::resolve_generic_string(ctx.source, key, args, "")),
    }
}
