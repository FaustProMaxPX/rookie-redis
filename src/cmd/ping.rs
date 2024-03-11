use tracing::instrument;

use crate::{Result, Connection, Frame};

#[derive(Debug)]
pub struct Ping;

impl Ping {
    #[instrument(skip(connection))]
    pub async fn execute(&self, connection: &mut Connection) -> Result<()> {
        connection.write_frame(Frame::into_simple("pong")).await 
    }

    pub fn get_frame() -> Frame {
        Frame::Array(vec![Frame::into_simple("ping")])
    }
}