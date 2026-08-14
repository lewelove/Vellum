mod core;
mod discogs;
mod fs;
mod models;
mod musicbrainz;

use anyhow::Result;
use libactions::payload::read_stdin_payload;

#[tokio::main]
async fn main() -> Result<()> {
    let payload: models::ActionPayload = read_stdin_payload()?;
    core::execute_collect(&payload).await?;
    Ok(())
}
