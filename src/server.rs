use crate::{connection::Connection, Frame, Result};
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
            match frame {
                Frame::Simple(command) => {
                    if command == "ping" {
                        self.connection
                            .write_frame(Frame::into_simple("pong"))
                            .await?;
                        Ok(())
                    } else {
                        self.connection
                            .write_frame(Frame::into_simple("unknown command"))
                            .await?;
                        Err("unknown command".into())
                    }
                }
                Frame::Error(error) => {
                    self.connection
                        .write_frame(Frame::into_simple(&error))
                        .await?;
                    Err(error.into())
                }
            }
        } else {
            self.connection
                .write_frame(Frame::into_simple("I don't know"))
                .await?;
            Ok(())
        }
    }
}
