mod core;
mod metrics;
mod models;

use anyhow::Result;
use libactions::payload::{read_stdin_payload, ActionPayload};

fn main() -> Result<()> {
    let payload: ActionPayload = read_stdin_payload()?;
    core::execute(&payload);
    Ok(())
}
