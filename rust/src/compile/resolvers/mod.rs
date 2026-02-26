pub mod native;
pub mod standard;

use crate::compile::builder::context::{AlbumContext, TrackContext};
use serde_json::{Value, json};

pub fn resolve_album_key(key: &str, ctx: &AlbumContext) -> Option<Value> {
    match key {
        "album" => Some(json!(standard::get_raw(
            ctx.source,
            "album",
            "Unknown Album"
        ))),
        "albumartist" => Some(json!(standard::get_raw(
            ctx.source,
            "albumartist",
            "Unknown Artist"
        ))),
        "date" => Some(json!(native::resolve_date(ctx))),
        "genre" => Some(json!(native::resolve_genre(ctx))),
        "comment" => Some(json!(native::resolve_comment(ctx))),
        "original_yyyy_mm" => Some(json!(native::resolve_yyyy_mm(ctx, "original_yyyy_mm"))),
        "release_yyyy_mm" => Some(json!(native::resolve_yyyy_mm(ctx, "release_yyyy_mm"))),
        "custom_albumartist" => Some(json!(native::resolve_custom_albumartist(ctx))),
        "cover_chroma" => native::resolve_cover_chroma(ctx),
        "cover_entropy" => native::resolve_cover_entropy(ctx),
        _ => None,
    }
}

pub fn resolve_track_key(key: &str, ctx: &TrackContext) -> Option<Value> {
    match key {
        "title" => Some(json!(standard::get_raw(ctx.source, "title", "Untitled"))),
        "artist" => Some(json!(standard::get_raw_with_fallback(
            ctx.source,
            ctx.album_source,
            "artist",
            "albumartist",
            "Unknown Artist"
        ))),
        "tracknumber" => Some(json!(ctx.ordinal_track_number)),
        "discnumber" => Some(json!(ctx.ordinal_disc_number)),
        _ => None,
    }
}
