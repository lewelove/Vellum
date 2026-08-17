mod core;
mod discogs;
mod models;
mod terminal;

use anyhow::Result;
use libactions::payload::{read_stdin_payload, ActionPayload};

#[tokio::main]
async fn main() -> Result<()> {
    let payload: ActionPayload<models::ActionConfig> = read_stdin_payload()?;
    core::execute(&payload).await?;
    Ok(())
}
