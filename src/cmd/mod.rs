mod ping;
use std::{sync::Arc, time::Duration};

pub use ping::Ping;

mod get;
pub use get::Get;

mod set;
use crate::{parser::ParseError, parser::Parser, Connection, DbHolder, Frame, Result};
pub use set::Set;

pub enum Command {
    Ping(Ping),
    Get(Get),
    Set(Set),
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command> {
        use ParseError::EndOfStream;

        let mut parser = Parser::new(frame)?;
        let cmd_name = parser.next_string()?;
        let cmd = match &cmd_name[..] {
            "ping" => Command::Ping(Ping {}),
            "get" => {
                let key = parser.next_string()?;
                Command::Get(Get::new(key))
            }
            "set" => {
                let key = parser.next_string()?;
                let value = parser.next_bytes()?;
                match parser.next_int() {
                    Ok(exp) => {
                        Command::Set(Set::new(key, value, Some(Duration::from_secs(exp as u64))))
                    }
                    Err(EndOfStream) => Command::Set(Set::new(key, value, None)),
                    Err(e) => return Err(e.into()),
                }
            }
            _ => return Err(format!("unrecognized command '{}'", cmd_name).into()),
        };
        parser.check_finished()?;
        Ok(cmd)
    }

    pub async fn execute(self, connection: &mut Connection, db: &Arc<DbHolder>) -> Result<()> {
        match self {
            Command::Ping(cmd) => cmd.execute(connection).await,
            Command::Get(cmd) => cmd.execute(connection, db).await,
            Command::Set(cmd) => cmd.execute(db, connection).await,
        }
    }
}
