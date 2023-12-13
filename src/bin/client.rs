use rookie_redis::Connection;
use rookie_redis::Frame;
use rookie_redis::Result;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("localhost:6379").await.unwrap();
    let mut connection = Connection::new(stream);
    connection
        .write_frame(Frame::Simple("ping".to_string()))
        .await?;
    let frame = connection.read_frame().await?;
    if let Some(Frame::Simple(frame)) = frame {
        println!("{}", frame);
    } else {
        println!("{:?}", frame.unwrap());
    }
    Ok(())
}
