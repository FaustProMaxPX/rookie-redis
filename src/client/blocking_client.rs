use crate::{Connection, Frame, Ping, Result};

struct BlockingClient {
    connection: Connection,
}

impl BlockingClient {
    pub fn new(connection: Connection) -> BlockingClient {
        BlockingClient { connection }
    }

    pub async fn ping(&mut self) -> Result<String> {
        let f = Ping::get_frame();
        self.connection.write_frame(f).await?;
        let resp = self.connection.read_frame().await?;

        if let Some(frame) = resp {
            match frame {
                Frame::Simple(s) => Ok(s),
                Frame::Bulk(s) => Ok(String::from_utf8_lossy(&s).to_string()),
                _frame => Err("There is some errors with server response".into()),
            }
        } else {
            Err("no response from server".into())
        }
    }
}

#[cfg(test)]
mod blocking_client_test {
    use tokio::net::TcpStream;

    use super::*;

    #[tokio::test]
    async fn ping_test() -> Result<()> {
        let conn = Connection::new(TcpStream::connect("localhost:6379").await?);
        let mut client = BlockingClient::new(conn);
        let resp = client.ping().await?;
        assert_eq!(resp, "pong");
        Ok(())
    }
}