mod server;
pub use server::{Listener, Handler};

mod connection;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;