use crate::Result;
use bytes::BytesMut;
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}, spawn};

pub struct Listener {
    listener: TcpListener,
}

pub struct Handler {
    connection: TcpStream,
}

impl Listener {
    pub async fn new(addr: &str) -> Result<Listener> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Listener { listener })
    }

    pub async fn accecpt(&self) -> Result<Handler> {
        let (connection, _) = self.listener.accept().await?;
        Ok(Handler::new(connection))
    }

    pub async fn run(&mut self) -> Result<()>{
        loop {
            let mut handler = self.accecpt().await?;
            spawn(async move {
                if let Err(e) = handler.run().await {
                    println!("{}", e);
                }
            });
        }
    }
}

impl Handler {
    fn new(connection: TcpStream) -> Handler {
        Handler { connection }
    }

    async fn run(&mut self) -> Result<()>{
        let mut buf = BytesMut::with_capacity(64);
        let _ = self.connection.read_buf(&mut buf).await?;
        let recv = String::from_utf8_lossy(&buf[..]);
        if recv == "ping" {
            self.connection.write_all("pong".as_bytes()).await?;
            Ok(())
        } else {
            self.connection.write_all("I don't know".as_bytes()).await?;
            Ok(())
        }
    }
}
