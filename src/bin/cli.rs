use std::{convert::Infallible, num::ParseIntError, time::Duration};

use bytes::Bytes;
use clap::{Parser, Subcommand};
use rookie_redis::BlockingClient;
use rookie_redis::Result;

#[derive(Parser)]
#[command(author = "Zephyr", version = "0.1.0", about = "rookie-redis client", long_about = None)]
pub struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(short, long, default_value = "6379")]
    port: u16,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Ping,

    Get {
        key: String,
    },

    Set {
        key: String,
        #[clap(value_parser = bytes_from_str)]
        value: Bytes,
        #[clap(value_parser = duration_from_str)]
        expiration: Option<Duration>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut client = BlockingClient::connect(&cli.host, cli.port).await?;
    match cli.command {
        Command::Ping => {
            let resp = client.ping().await?;
            println!("{}", resp);
        }
        Command::Get { key } => {
            let resp = client.get(&key).await?;
            if let Some(resp) = resp {
                if let Ok(resp) = std::str::from_utf8(&resp) {
                    println!("{}", resp);
                } else {
                    println!("{:?}", resp);
                }
            } else {
                println!("nil");
            }
        }
        Command::Set {
            key,
            value,
            expiration,
        } => {
            client.set(&key, value, expiration).await?;
            println!("OK")
        }
    }

    Ok(())
}

fn bytes_from_str(s: &str) -> std::result::Result<Bytes, Infallible> {
    Ok(Bytes::copy_from_slice(s.as_bytes()))
}

fn duration_from_str(s: &str) -> std::result::Result<Duration, ParseIntError> {
    let ms = s.parse::<u64>()?;
    Ok(Duration::from_millis(ms))
}
