use anyhow::Result;
use mpd::Client;
use std::net::TcpStream;

pub enum MpdCommand {
    Play { tracks: Vec<String>, offset: usize },
    Queue { tracks: Vec<String> },
    Clear,
    Stop,
    Next,
    Prev,
    TogglePause,
    Refresh,
}

pub fn handle_command(client: &mut Client<TcpStream>, cmd: MpdCommand) -> Result<()> {
    match cmd {
        MpdCommand::Play { tracks, offset } => {
            client.clear()?;
            for track in tracks {
                client.push(mpd::song::Song {
                    file: track,
                    ..Default::default()
                })?;
            }
            client.switch(u32::try_from(offset)?)?;
        }
        MpdCommand::Queue { tracks } => {
            for track in tracks {
                client.push(mpd::song::Song {
                    file: track,
                    ..Default::default()
                })?;
            }
        }
        MpdCommand::Clear => client.clear()?,
        MpdCommand::Stop => client.stop()?,
        MpdCommand::Next => client.next()?,
        MpdCommand::Prev => client.prev()?,
        MpdCommand::TogglePause => {
            let status = client.status()?;
            match status.state {
                mpd::status::State::Play => client.pause(true)?,
                mpd::status::State::Pause => client.pause(false)?,
                mpd::status::State::Stop => client.play()?,
            }
        }
        MpdCommand::Refresh => {}
    }
    Ok(())
}
