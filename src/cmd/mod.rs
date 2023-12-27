mod ping;
use std::sync::Arc;

pub use ping::Ping;

mod get;
pub use get::Get;

use crate::{parser::Parser, Frame, Result, Connection, DbHolder};

pub enum Command {
    Ping(Ping),
    Get(Get),
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command> {
        let mut parser = Parser::new(frame)?;
        let cmd_name = parser.next_string()?;
        let cmd = match &cmd_name[..] {
            "ping" => Command::Ping(Ping {}),
            "get" => {
                let key = parser.next_string()?;
                Command::Get(Get::new(&key))
            }
            _ => return Err(format!("unrecognized command '{}'", cmd_name).into()),
        };
        parser.check_finished()?;
        Ok(cmd)
    }

    pub async fn execute(&self, connection: &mut Connection, db: &Arc<DbHolder>) -> Result<()> {
        match self {
            Command::Ping(cmd) => cmd.execute(connection).await, 
            Command::Get(cmd) => cmd.execute(connection, db).await,
        }
    }
}
