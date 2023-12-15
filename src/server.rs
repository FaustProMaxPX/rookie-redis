use std::sync::Arc;

use crate::{cmd::Command, connection::Connection, Error, Frame, Result};
use tokio::{
    net::{TcpListener, TcpStream},
    spawn,
    sync::Semaphore,
};

const MAX_LIMIT_CONNECTIONS: usize = 1;

pub struct Listener {
    listener: TcpListener,
    semaphore: Arc<Semaphore>,
}

pub struct Handler {
    connection: Connection,
}

impl Listener {
    pub async fn new(addr: &str) -> Result<Listener> {
        let listener = TcpListener::bind(addr).await?;
        let semaphore = Arc::new(Semaphore::new(MAX_LIMIT_CONNECTIONS));
        Ok(Listener {
            listener,
            semaphore,
        })
    }

    pub async fn accecpt(&self) -> Result<Handler> {
        let (connection, _) = self.listener.accept().await?;
        Ok(Handler::new(connection))
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();

            let mut handler = self.accecpt().await?;
            spawn(async move {
                if let Err(e) = handler.run().await {
                    println!("{}", e);
                    handler.send_error_msg(e).await.unwrap();
                }

                drop(permit);
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
        // TODO: break out the loop when the server shutdown
        loop {
            let frame = self.connection.read_frame().await?;
            if let Some(frame) = frame {
                let cmd = Command::from_frame(frame)?;
                cmd.execute(&mut self.connection).await?;
            } else {
                // this means that the client has closed the connection
                return Ok(());
            }
        }
    }

    async fn send_error_msg(&mut self, e: Error) -> Result<()> {
        self.connection
            .write_frame(Frame::into_simple(&format!("error: {}", e)))
            .await
    }
}
