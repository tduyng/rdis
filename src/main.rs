use anyhow::Result;
use redis_starter_rust::{command::CommandRegistry, stream::handle_stream};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server listening on 127.0.0.1:6379");

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
