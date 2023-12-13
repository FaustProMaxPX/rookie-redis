use crate::{connection::Connection, Result};
use tokio::{
    net::{TcpListener, TcpStream},
    spawn,
};

pub struct Listener {
    listener: TcpListener,
}

pub struct Handler {
    connection: Connection,
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

    pub async fn run(&mut self) -> Result<()> {
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
        Handler {
            connection: Connection::new(connection),
        }
    }

    async fn run(&mut self) -> Result<()> {
        let frame = self.connection.read_frame().await?;
        if let Some(frame) = frame {
            if frame == *"ping" {
                self.connection.write_frame("pong").await?;
                Ok(())
            } else {
                self.connection.write_frame("unknown command").await?;
                Err("unknown command".into())
            }
        } else {
            self.connection.write_frame("I don't know").await?;
            Ok(())
        }
    }
}
