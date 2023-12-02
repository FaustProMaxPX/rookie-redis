use rookie_redis::{Listener, Result};

const DEFAULT_ADDR: &str = "127.0.0.1:6379";

#[tokio::main]
async fn main() -> Result<()>{
    let mut listener = Listener::new(DEFAULT_ADDR).await?;
    println!("server has started");
    listener.run().await?;
    Ok(())
}