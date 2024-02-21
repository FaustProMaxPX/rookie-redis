use std::time::Duration;

use bytes::Bytes;
use tokio::net::TcpStream;

use crate::{Connection, Frame, Get, Ping, Result, Set};

use super::parse_frame;

pub struct BlockingClient {
    connection: Connection,
}

impl BlockingClient {
    pub fn new(connection: Connection) -> BlockingClient {
        BlockingClient { connection }
    }

    pub async fn connect(host: &str, port: u16) -> Result<BlockingClient> {
        let conn = Connection::new(TcpStream::connect(format!("{}:{}", host, port)).await?);
        Ok(Self::new(conn))
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

    pub async fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        let f = Get::get_frame(key);
        self.connection.write_frame(f).await?;
        let resp = self.connection.read_frame().await?;
        if resp.is_none() {
            return Ok(None);
        }
        parse_frame(resp.unwrap())
    }

    pub async fn set(
        &mut self,
        key: &str,
        value: Bytes,
        expiration: Option<Duration>,
    ) -> Result<()> {
        let f = Set::get_frame(key, value, expiration);
        println!("{:?}", f);
        self.connection.write_frame(f).await?;
        let resp = self.connection.read_frame().await?;
        if let Some(frame) = resp {
            match frame {
                Frame::Simple(_) => Ok(()),
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
