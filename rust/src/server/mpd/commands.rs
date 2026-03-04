use anyhow::Result;
use mpd_client::Client;
use mpd_client::commands::{
    Add, 
    Play, 
    Stop, 
    Next, 
    Previous, 
    SetPause,
    Status,
    SongPosition,
};
use mpd_client::protocol::command::Command as RawCommand;
use mpd_client::responses::PlayState;

pub enum MpdCommand {
    Play { 
        tracks: Vec<String>, 
        offset: usize 
    },
    Queue { 
        tracks: Vec<String> 
    },
    Clear,
    Stop,
    Next,
    Prev,
    TogglePause,
    Refresh,
}

pub async fn handle_command(client: &Client, cmd: MpdCommand) -> Result<()> {
    match cmd {
        MpdCommand::Play { tracks, offset } => {
            client.raw_command(RawCommand::new("clear")).await?;
            for track in &tracks {
                client.command(Add::uri(track)).await?;
            }
            client.command(Play::song(SongPosition(offset))).await?;
        }
        MpdCommand::Queue { tracks } => {
            for track in &tracks {
                client.command(Add::uri(track)).await?;
            }
        }
        MpdCommand::Clear => {
            client.raw_command(RawCommand::new("clear")).await?;
        }
        MpdCommand::Stop => {
            client.command(Stop).await?;
        }
        MpdCommand::Next => {
            client.command(Next).await?;
        }
        MpdCommand::Prev => {
            client.command(Previous).await?;
        }
        MpdCommand::TogglePause => {
            let status = client.command(Status).await?;
            match status.state {
                PlayState::Playing => {
                    client.command(SetPause(true)).await?;
                }
                PlayState::Paused => {
                    client.command(SetPause(false)).await?;
                }
                PlayState::Stopped => {
                    client.command(Play::current()).await?;
                }
            }
        }
        MpdCommand::Refresh => {}
    }
    Ok(())
}
