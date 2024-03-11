use std::{future::Future, sync::Arc};

use crate::{cmd::Command, connection::Connection, DbHolder, Error, Frame, Result};
use tokio::{
    net::TcpListener,
    select, spawn,
    sync::{broadcast, mpsc, Semaphore},
};
use tracing::instrument;

const MAX_LIMIT_CONNECTIONS: usize = 10;

pub struct Listener {
    listener: TcpListener,
    semaphore: Arc<Semaphore>,
    shutdown_broadcast: broadcast::Sender<()>,
    shutdown_completed_tx: mpsc::Sender<()>,
}

pub struct Handler {
    connection: Connection,
    db: DbHolder,
    shutdown_receiver: broadcast::Receiver<()>,
    _shutdown_completed_tx: mpsc::Sender<()>,
}

pub async fn run(addr: &str, signal: impl Future) -> Result<()> {
    let (shutdown_tx, _) = broadcast::channel(1);
    let (shutdown_completed_tx, mut shutdown_completed_rx) = mpsc::channel(1);

    let mut listener = Listener::new(addr, shutdown_tx, shutdown_completed_tx).await?;

    select! {
        res = listener.run() => {
            if let Err(e) = res {
                eprintln!("server error: {}", e);
            }
        }
        _ = signal => {
            println!("shutdown");
        }
    }

    println!("send signal");
    listener.shutdown_broadcast.send(()).unwrap();
    let Listener {
        shutdown_completed_tx,
        ..
    } = listener;
    drop(shutdown_completed_tx);
    shutdown_completed_rx.recv().await;
    println!("shutdown has completed");

    Ok(())
}

impl Listener {
    pub async fn new(
        addr: &str,
        shutdown_broadcast: broadcast::Sender<()>,
        shutdown_completed_tx: mpsc::Sender<()>,
    ) -> Result<Listener> {
        let listener = TcpListener::bind(addr).await?;
        let semaphore = Arc::new(Semaphore::new(MAX_LIMIT_CONNECTIONS));
        Ok(Listener {
            listener,
            semaphore,
            shutdown_broadcast,
            shutdown_completed_tx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let db = DbHolder::new(
            self.shutdown_broadcast.subscribe(),
            self.shutdown_completed_tx.clone(),
        );
        loop {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();

            let (socket, _) = self.listener.accept().await?;
            let mut handler = Handler {
                connection: Connection::new(socket),
                db: db.clone(),
                shutdown_receiver: self.shutdown_broadcast.subscribe(),
                _shutdown_completed_tx: self.shutdown_completed_tx.clone(),
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
    #[instrument(skip(self))]
    async fn run(&mut self) -> Result<()> {
        loop {
            let frame = select! {
                frame = self.connection.read_frame() => frame?,
                _ = self.shutdown_receiver.recv() => {
                    return Err("server has been closed".into());
                },
            };

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
