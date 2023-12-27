use std::time::Duration;

use rookie_redis::Connection;
use rookie_redis::Frame;
use rookie_redis::Get;
use rookie_redis::Result;
use tokio::net::TcpStream;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("localhost:6379").await.unwrap();
    let mut connection = Connection::new(stream);
    connection
        .write_frame(Get::get_frame("hello"))
        .await?;
    let frame = connection.read_frame().await?;
    if let Some(Frame::Simple(frame)) = frame {
        println!("{}", frame);
    } else {
        println!("{:?}", frame.unwrap());
    }
    sleep(Duration::from_secs(10)).await;
    Ok(())
}
