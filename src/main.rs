use anyhow::{Context, Result};
use clap::Parser;
use redis_starter_rust::{
    replica::handshake::perform_hashshake,
    store::RedisStore,
    stream::{RespHandler, StreamInfo, StreamType},
    utils::random_sha1_hex,
};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::RwLock};

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "6379")]
    port: u16,
    #[clap(short, long = "replicaof", value_names = &["MASTER_HOST", "MASTER_PORT"], num_args = 2)]
    replica: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port))
        .await
        .context("Failed to bind to address")?;
    println!("Server listening on 127.0.0.1:{}", args.port);

    let role = match &args.replica {
        None => StreamType::Master,
        Some(replica_args) => {
            let replica_args_clone = replica_args.clone();
            let result =
                tokio::spawn(async move { perform_hashshake(&replica_args_clone).await }).await?;
            if let Err(e) = result {
                eprintln!("Error performing replica handshake: {:?}", e);
                return Err(e);
            }
            StreamType::Slave
        }
    };
    let stream_info = StreamInfo {
        role,
        master_id: random_sha1_hex(),
        master_offset: 0,
    };
    let store = Arc::new(RwLock::new(RedisStore::new()));

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .context("Failed to accept incoming connection")?;
        println!("Accepted new connection");
        let store_clone = Arc::clone(&store);
        let stream_info = stream_info.clone();
        tokio::spawn(async move {
            if let Err(e) = RespHandler::handle_stream(stream, &store_clone, stream_info).await {
                eprintln!("Error handling stream: {:?}", e);
            }
        });
    }
}
