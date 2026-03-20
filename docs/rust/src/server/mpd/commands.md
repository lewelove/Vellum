File: rust/src/server/mpd/commands.rs
Role: Player Command Translator

Description:
This file defines the vocabulary of actions the application can take and strictly translates those high-level intents into raw protocols that the music daemon understands.

Imports:
`use mpd_client::commands::{...};`
- The specific raw actions the daemon recognizes.
`use mpd_client::protocol::command::{Command as RawCommand, CommandList as RawCommandList};`
- Elements to build batched instructions.

Logic:
`pub enum MpdCommand`
- A clean list of actions requested by the UI, acting as a buffer between the web layer and the audio layer.

`pub async fn handle_command(client: &Client, cmd: MpdCommand) -> Result<()>`
- Executes the intents.
- When instructed to play an album, it doesn't just send individual commands because that would cause glitches. Instead, it builds an atomic `RawCommandList`. It clears the queue, queues every specific track sequentially, and executes the play command at a specific offset track all in a single lightning-fast transaction. Other simpler commands (Stop, Next, Prev) are handed directly to the daemon. For play/pause toggling, it queries the exact current state and sends the inverse command.
