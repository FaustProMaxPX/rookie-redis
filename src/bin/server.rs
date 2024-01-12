use rookie_redis::{Result, server};
use tokio::signal::ctrl_c;

const DEFAULT_ADDR: &str = "127.0.0.1:6379";

#[tokio::main]
async fn main() -> Result<()>{
    server::run(DEFAULT_ADDR, ctrl_c()).await?;
    Ok(())
}