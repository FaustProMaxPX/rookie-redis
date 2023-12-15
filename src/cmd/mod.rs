mod ping;
pub use ping::Ping;

use crate::{parser::Parser, Frame, Result};

pub enum Command {
    Ping(Ping),
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command> {
        let mut parser = Parser::new(frame)?;
        let cmd_name = parser.next_string()?;
        let cmd = match &cmd_name[..] {
            "ping" => Command::Ping(Ping {}),
            _ => return Err(format!("unrecognized command '{}'", cmd_name).into()),
        };
        parser.check_finished()?;
        Ok(cmd)
    }

    pub async fn execute(&self, connection: &mut crate::Connection) -> Result<()> {
        match self {
            Command::Ping(cmd) => cmd.execute(connection).await, 
        }
    }
}
