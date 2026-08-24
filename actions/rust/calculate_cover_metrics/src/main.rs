mod core;
mod metrics;
mod models;

use anyhow::Result;
use libactions::payload::{ActionPayload, read_stdin_payload};

fn main() -> Result<()> {
    let payload: ActionPayload = read_stdin_payload()?;
    core::execute(&payload);
    Ok(())
}
