use crate::fs;
use crate::models::{ActionPayload, AlbumData, Track};
use anyhow::{Context, Result};
use libactions::discogs::{
    TargetUrl, fetch_discogs_master, fetch_discogs_release, format_artist_credits,
    parse_discogs_position, parse_discogs_url,
};
use libactions::paths::expand_path;

pub async fn execute_collect(payload: &ActionPayload) -> Result<()> {
    let target = parse_discogs_url(&payload.options)
        .context("Invalid or unsupported URL provided")?;

    let mut album_data = AlbumData::default();

    match target {
        TargetUrl::DiscogsMaster(id) => {
            fetch_and_fill_discogs_master(id, &mut album_data).await?;
        }
        TargetUrl::DiscogsRelease(id) => {
            let release = fetch_discogs_release(id).await?;
            if let Some(master_id) = release.master_id {
                fetch_and_fill_discogs_master(master_id, &mut album_data).await?;
            } else {
                fetch_and_fill_discogs_release(id, release, &mut album_data)?;
            }
        }
    }

    if album_data.albumartist.is_empty() || album_data.album.is_empty() {
        anyhow::bail!("Failed to fetch sufficient album data");
    }

    let root_expanded = expand_path(&payload.config.action.root);
    fs::create_album_directory(
        &album_data,
        &payload.config.action.formatting,
        &root_expanded,
    )
    .await?;

    Ok(())
}

async fn fetch_and_fill_discogs_master(id: u64, data: &mut AlbumData) -> Result<()> {
    let master = fetch_discogs_master(id).await?;
    data.discogs_raw = Some(serde_json::to_value(&master)?);
    data.discogs_master_id = Some(id.to_string());
    data.album = master.title;
    data.date = master.year.map_or_else(String::new, |y| y.to_string());

    if let Some(artists) = master.artists {
        data.albumartist = format_artist_credits(&artists);
    }

    data.genre = master.genres.unwrap_or_default();
    data.styles = master.styles.unwrap_or_default();

    if let Some(images) = master.images {
        data.discogs_cover_url = images
            .into_iter()
            .find(|img| img.image_type == "primary")
            .map(|img| img.uri);
    }

    let mut tracks = Vec::new();
    if let Some(tracklist) = master.tracklist {
        let mut d_counter = 1;
        let mut t_counter = 0;
        for t in tracklist {
            if let Some(pos) = t.position {
                if pos.is_empty() {
                    continue;
                }
                let (disc, track) = parse_discogs_position(&pos, &mut d_counter, &mut t_counter);
                let mut track_artist = None;
                if let Some(artists_val) = t.extra.get("artists")
                    && let Ok(artists) = serde_json::from_value::<Vec<discogs_rs::ArtistCredit>>(
                        artists_val.clone(),
                    )
                {
                    let parsed_art = format_artist_credits(&artists);
                    if !parsed_art.is_empty() {
                        track_artist = Some(parsed_art);
                    }
                }
                tracks.push(Track {
                    discnumber: disc,
                    tracknumber: track,
                    title: t.title,
                    artist: track_artist,
                });
            }
        }
    }
    data.tracks = tracks;

    Ok(())
}

fn fetch_and_fill_discogs_release(
    id: u64,
    release: discogs_rs::Release,
    data: &mut AlbumData,
) -> Result<()> {
    data.discogs_raw = Some(serde_json::to_value(&release)?);
    data.discogs_master_id = Some(id.to_string());
    data.album = release.title;
    data.date = release.year.map_or_else(String::new, |y| y.to_string());

    if let Some(artists) = release.artists {
        data.albumartist = format_artist_credits(&artists);
    }

    data.genre = release.genres.unwrap_or_default();
    data.styles = release.styles.unwrap_or_default();

    if let Some(images) = release.images {
        data.discogs_cover_url = images
            .into_iter()
            .find(|img| img.image_type == "primary")
            .map(|img| img.uri);
    }

    let mut tracks = Vec::new();
    if let Some(tracklist) = release.tracklist {
        let mut d_counter = 1;
        let mut t_counter = 0;
        for t in tracklist {
            if let Some(pos) = t.position {
                if pos.is_empty() {
                    continue;
                }
                let (disc, track) = parse_discogs_position(&pos, &mut d_counter, &mut t_counter);
                let mut track_artist = None;
                if let Some(artists_val) = t.extra.get("artists")
                    && let Ok(artists) = serde_json::from_value::<Vec<discogs_rs::ArtistCredit>>(
                        artists_val.clone(),
                    )
                {
                    let parsed_art = format_artist_credits(&artists);
                    if !parsed_art.is_empty() {
                        track_artist = Some(parsed_art);
                    }
                }
                tracks.push(Track {
                    discnumber: disc,
                    tracknumber: track,
                    title: t.title,
                    artist: track_artist,
                });
            }
        }
    }
    data.tracks = tracks;

    Ok(())
}
