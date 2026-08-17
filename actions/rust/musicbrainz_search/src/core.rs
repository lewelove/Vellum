use anyhow::{Context, Result};
use libactions::discogs::{
    fetch_discogs_master, fetch_discogs_release, format_artist_credits, parse_discogs_url,
    TargetUrl,
};
use libactions::payload::ActionPayload;
use std::process::Command;

pub async fn execute(payload: &ActionPayload) -> Result<()> {
    let trimmed_options = payload.options.trim();

    if let Some(target) = parse_discogs_url(trimmed_options) {
        let (artist, title) = match target {
            TargetUrl::DiscogsMaster(id) => {
                let master = fetch_discogs_master(id).await?;
                let artist = master
                    .artists
                    .as_ref()
                    .map_or_else(String::new, |a| format_artist_credits(a));
                (artist, master.title)
            }
            TargetUrl::DiscogsRelease(id) => {
                let release = fetch_discogs_release(id).await?;
                if let Some(master_id) = release.master_id {
                    let master = fetch_discogs_master(master_id).await?;
                    let artist = master
                        .artists
                        .as_ref()
                        .map_or_else(String::new, |a| format_artist_credits(a));
                    (artist, master.title)
                } else {
                    let artist = release
                        .artists
                        .as_ref()
                        .map_or_else(String::new, |a| format_artist_credits(a));
                    (artist, release.title)
                }
            }
        };

        if !artist.is_empty() || !title.is_empty() {
            search_and_open(&artist, &title)?;
        }
    } else if !payload.albums.is_empty() {
        for item in &payload.albums {
            let album_lock = &item.lock;
            let Some(album_obj) = album_lock.get("album").and_then(serde_json::Value::as_object) else {
                continue;
            };

            let artist = album_obj
                .get("albumartist")
                .or_else(|| album_obj.get("artist"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");

            let title = album_obj
                .get("album")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");

            if !artist.is_empty() || !title.is_empty() {
                search_and_open(artist, title)?;
            }
        }
    } else if !trimmed_options.is_empty() {
        if let Some((artist, title)) = trimmed_options.split_once(" - ") {
            search_and_open(artist.trim(), title.trim())?;
        } else {
            open_query(trimmed_options)?;
        }
    }

    Ok(())
}

fn search_and_open(artist: &str, title: &str) -> Result<()> {
    let query = if artist.is_empty() {
        format!("releasegroup:\"{title}\"")
    } else if title.is_empty() {
        format!("artist:\"{artist}\"")
    } else {
        format!("artist:\"{artist}\" AND releasegroup:\"{title}\"")
    };

    open_query(&query)
}

fn open_query(query: &str) -> Result<()> {
    let encoded = urlencoding::encode(query);
    let url = format!("https://musicbrainz.org/search?type=release_group&method=advanced&query={encoded}");

    let launcher = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };

    Command::new(launcher)
        .arg(&url)
        .spawn()
        .context(format!("Failed to open MusicBrainz search URL: {url}"))?;

    Ok(())
}
