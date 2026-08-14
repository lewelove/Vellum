mod core;
mod metrics;
mod models;

use anyhow::Result;
use libactions::payload::read_stdin_payload;

fn main() -> Result<()> {
    let payload: models::ActionPayload = read_stdin_payload()?;
    core::execute(&payload)?;
    Ok(())
}
