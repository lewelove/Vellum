use crate::discogs;
use crate::fs;
use crate::models::{ActionPayload, AlbumData};
use crate::musicbrainz;
use anyhow::Result;
use libactions::paths::expand_path;

pub async fn execute_collect(payload: &ActionPayload) -> Result<()> {
    let opts = payload.options.trim();
    let mut album_data = AlbumData::default();

    if let Some(mb_target) = musicbrainz::parse_musicbrainz_url(opts) {
        musicbrainz::execute_musicbrainz(mb_target, &mut album_data).await?;
    } else if let Some(discogs_target) = discogs::parse_discogs_url(opts) {
        discogs::execute_discogs(discogs_target, &mut album_data).await?;
    } else {
        anyhow::bail!("Invalid or unsupported URL provided: '{opts}'");
    }

    if album_data.albumartist.is_empty() || album_data.album.is_empty() {
        anyhow::bail!("Failed to fetch sufficient album data");
    }

    let root_expanded = expand_path(&payload.config.action.root);
    fs::create_album_directory(
        &album_data,
        &payload.config.action.formatting,
        &root_expanded,
    )?;

    Ok(())
}
