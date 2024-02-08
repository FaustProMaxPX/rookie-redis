use clap::Parser;

mod blocking_client;

#[derive(Parser)]
#[command(author = "Zephyr", version = "0.1.0", about = "rookie-redis client", long_about = None)]
struct Client {
    
}
