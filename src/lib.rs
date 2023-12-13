mod server;
pub use server::{Handler, Listener};

mod connection;
pub use connection::Connection;

mod frame;
pub use frame::Frame;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
