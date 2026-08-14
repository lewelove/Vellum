mod core;
mod models;

use anyhow::Result;
use libactions::payload::read_stdin_payload;

#[tokio::main]
async fn main() -> Result<()> {
    let payload: models::ActionPayload = read_stdin_payload()?;
    core::execute(&payload).await?;
    Ok(())
}
