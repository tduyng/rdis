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
use tokio::{net::TcpListener, sync::Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    let stream_info = Arc::new(Mutex::new(StreamInfo::new(&args)));
    let store = Arc::new(Mutex::new(RedisStore::new()));

    if should_replicate(&stream_info).await {
        let info = stream_info.clone();
        let store = store.clone();
        tokio::spawn(async move {
            let stream = perform_handshake_to_master(&info)
                .await
                .expect("Failed the handshake with the master");
            let _ = Handler::handle_replica(stream, store).await;
        });
    }

    let socket_addr = stream_info.lock().await.socket_addr;
    let listener = TcpListener::bind(socket_addr)
        .await
        .context("failed to bind to address")?;
    println!("Server listening on {}", socket_addr);

    loop {
        let stream_info = stream_info.clone();
        let store = store.clone();

        let (stream, _) = listener
            .accept()
            .await
            .context("failed to accept incoming connection")?;
        println!("Accepted new connection");

        tokio::spawn(async move {
            let _ = Handler::handle_stream(stream, store, stream_info).await;
        });
    }
}
