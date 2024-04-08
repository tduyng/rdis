use anyhow::Result;
use clap::Parser;
use redis_starter_rust::{command::CommandRegistry, stream::handle_stream};
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "6379")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    println!("Server listening on 127.0.0.1:{}", args.port);

    let registry = CommandRegistry::new();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_stream(stream, registry.clone()));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
