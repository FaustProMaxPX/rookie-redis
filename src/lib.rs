mod server;
pub use server::{Handler, Listener};

mod connection;
pub use connection::Connection;

mod frame;
pub use frame::Frame;

mod cmd;
pub use cmd::{Command, Ping};

mod parser;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
