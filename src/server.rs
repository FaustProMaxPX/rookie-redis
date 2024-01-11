use std::sync::Arc;

use crate::{cmd::Command, connection::Connection, DbHolder, Error, Frame, Result};
use tokio::{
    net::TcpListener,
    spawn,
    sync::Semaphore,
};

const MAX_LIMIT_CONNECTIONS: usize = 10;

pub struct Listener {
    listener: TcpListener,
    semaphore: Arc<Semaphore>,
}

pub struct Handler {
    connection: Connection,
    db: DbHolder,
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

    pub async fn run(&mut self) -> Result<()> {
        let db = DbHolder::new();
        loop {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();

            let (socket, _) = self.listener.accept().await?;
            let mut handler = Handler {
                connection: Connection::new(socket),
                db: db.clone(),
            };

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
    async fn run(&mut self) -> Result<()> {
        // TODO: break out the loop when the server shutdown
        loop {
            let frame = self.connection.read_frame().await?;
            if let Some(frame) = frame {
                let cmd = Command::from_frame(frame)?;
                cmd.execute(&mut self.connection, &self.db).await?;
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
