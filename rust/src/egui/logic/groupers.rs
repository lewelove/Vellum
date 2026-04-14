use crate::server::library::models::AlbumView;
use std::collections::HashMap;

pub struct Bucket {
    pub label: String,
    pub value: String,
    pub count: usize,
    pub filter_target: String,
}

pub fn generate_buckets(albums: &[&AlbumView], group_key: &str) -> Vec<Bucket> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for album in albums {
        match group_key {
            "genre" => {
                for g in &album.album_data.genre {
                    *counts.entry(g.clone()).or_insert(0) += 1;
                }
            }
            "decade" => {
                if album.album_data.date.len() >= 4 {
                    let d = format!("{}0s", &album.album_data.date[0..3]);
                    *counts.entry(d).or_insert(0) += 1;
                }
            }
            "year_added" => {
                let year = chrono::DateTime::from_timestamp(album.album_data.info.unix_added as i64, 0)
                    .map(|dt| dt.format("%Y").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                *counts.entry(year).or_insert(0) += 1;
            }
            "chroma" => {
                let score = album.album_data.tags.get("COVER_CHROMA")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let label = if score == 0.0 { "Monochrome" }
                    else if score < 15.0 { "Bleak" }
                    else if score < 33.0 { "Muted" }
                    else if score < 60.0 { "Standard" }
                    else { "Vibrant" };
                *counts.entry(label.to_string()).or_insert(0) += 1;
            }
            _ => {}
        }
    }

    let mut result: Vec<Bucket> = counts.into_iter().map(|(val, count)| Bucket {
        label: val.clone(),
        value: val,
        count,
        filter_target: group_key.to_string(),
    }).collect();

    match group_key {
        "chroma" => {
            let order = ["Vibrant", "Standard", "Muted", "Bleak", "Monochrome"];
            result.sort_by(|a, b| {
                let pos_a = order.iter().position(|&x| x == a.label).unwrap_or(99);
                let pos_b = order.iter().position(|&x| x == b.label).unwrap_or(99);
                pos_a.cmp(&pos_b)
            });
        }
        "year_added" | "decade" => result.sort_by(|a, b| b.value.cmp(&a.value)),
        _ => result.sort_by(|a, b| a.value.cmp(&b.value)),
    }

    result
}
