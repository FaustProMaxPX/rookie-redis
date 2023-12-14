use crate::{Result, Connection, Frame};
pub struct Ping;

impl Ping {
    pub async fn execute(&self, connection: &mut Connection) -> Result<()> {
        connection.write_frame(Frame::into_simple("pong")).await 
    }

    pub fn get_frame() -> Frame {
        Frame::Array(vec![Frame::into_simple("ping")])
    }
}