use crate::server::library::models::AlbumView;
use std::cmp::Ordering;

fn get_sortable_artist(name: &str) -> String {
    let n = name.trim();
    if n.to_lowercase().starts_with("the ") {
        n[4..].trim().to_lowercase()
    } else {
        n.to_lowercase()
    }
}

pub fn sort_default(a: &AlbumView, b: &AlbumView) -> Ordering {
    let artist_a = get_sortable_artist(&a.album_data.albumartist);
    let artist_b = get_sortable_artist(&b.album_data.albumartist);
    let artist_cmp = artist_a.cmp(&artist_b);
    if artist_cmp != Ordering::Equal { return artist_cmp; }

    let date_a = &a.album_data.date;
    let date_b = &b.album_data.date;
    let date_cmp = date_a.cmp(date_b);
    if date_cmp != Ordering::Equal { return date_cmp; }

    a.album_data.album.to_lowercase().cmp(&b.album_data.album.to_lowercase())
}

pub fn sort_date_added(a: &AlbumView, b: &AlbumView) -> Ordering {
    b.album_data.info.unix_added.cmp(&a.album_data.info.unix_added)
}

#[allow(dead_code)]
pub fn sort_alphabetical(a: &AlbumView, b: &AlbumView) -> Ordering {
    a.album_data.album.to_lowercase().cmp(&b.album_data.album.to_lowercase())
}
