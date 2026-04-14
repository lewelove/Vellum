use crate::server::library::models::AlbumView;

pub fn apply_filter(album: &AlbumView, key: &str, val: &str) -> bool {
    match key {
        "genre" => {
            album.album_data.genre.iter().any(|g| g.to_lowercase() == val.to_lowercase())
        }
        "decade" => {
            if album.album_data.date.len() < 4 { return false; }
            let year_str = &album.album_data.date[0..4];
            let start_str = &val[0..4];
            if let (Ok(year), Ok(start)) = (year_str.parse::<u32>(), start_str.parse::<u32>()) {
                year >= start && year <= start + 9
            } else {
                false
            }
        }
        "year_added" => {
            let year = chrono::DateTime::from_timestamp(album.album_data.info.unix_added as i64, 0)
                .map(|dt| dt.format("%Y").to_string())
                .unwrap_or_default();
            year == val
        }
        "chroma" => {
            let score = album.album_data.tags.get("COVER_CHROMA")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            match val {
                "Monochrome" => score == 0.0,
                "Bleak" => score > 0.0 && score < 15.0,
                "Muted" => score >= 15.0 && score < 33.0,
                "Standard" => score >= 33.0 && score < 60.0,
                "Vibrant" => score >= 60.0,
                _ => true,
            }
        }
        "search" => {
            let q = val.to_lowercase();
            if album.album_data.album.to_lowercase().contains(&q) || 
               album.album_data.albumartist.to_lowercase().contains(&q) {
                return true;
            }
            album.tracks.iter().any(|t| t.title.to_lowercase().contains(&q))
        }
        _ => true,
    }
}
