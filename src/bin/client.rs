
use bytes::Bytes;
use rookie_redis::Connection;
use rookie_redis::Frame;
use rookie_redis::Result;
use rookie_redis::{Get, Set};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("localhost:6379").await.unwrap();
    let mut connection = Connection::new(stream);
    connection
        .write_frame(Set::get_frame("foo", Bytes::copy_from_slice(b"bar"), None))
        .await?;
    read_resp(&mut connection).await?;
    connection.write_frame(Get::get_frame("foo")).await?;
    read_resp(&mut connection).await?;
    Ok(())
    
}

async fn read_resp(connection: &mut Connection) -> Result<()> {
    let frame = connection.read_frame().await?;
    if let Some(Frame::Simple(frame)) = frame {
        println!("{}", frame);
    } else {
        println!("{:?}", frame.unwrap());
    }
    Ok(())
}
