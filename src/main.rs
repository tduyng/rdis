use anyhow::{Context, Result};
use clap::Parser;
use redis_starter_rust::{
    args::CliArgs,
    handler::Handler,
    replica::{handshake::perform_handshake_to_master, should_replicate},
    store::RedisStore,
    stream::StreamInfo,
};
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{Mutex, RwLock},
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    let stream_info = Arc::new(Mutex::new(StreamInfo::new(&args)));

    let socket_addr = {
        let info = stream_info.lock().await;
        if should_replicate(&info) {
            let info = info.clone();
            tokio::spawn(async move {
                if let Err(e) = perform_handshake_to_master(&info).await {
                    eprintln!("error performing handshake to master: {}", e)
                }
            });
        }
        info.socket_addr
    };

    let store = Arc::new(RwLock::new(RedisStore::new()));
    let listener = TcpListener::bind(socket_addr)
        .await
        .context("Failed to bind to address")?;
    println!("Server listening on {}", socket_addr);

    loop {
        let stream_info = stream_info.clone();
        let store = store.clone();

        let (stream, _) = listener
            .accept()
            .await
            .context("Failed to accept incoming connection")?;
        println!("Accepted new connection");

        tokio::spawn(async move {
            let _ = Handler::handle_stream(stream, stream_info, store).await;
        });
    }
}
