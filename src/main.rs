use anyhow::{Context, Result};
use clap::Parser;
use redis_starter_rust::{
    replica::handshake::perform_handshake,
    store::RedisStore,
    stream::{RespHandler, StreamInfo, StreamType},
    utils::random_sha1_hex,
};
use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

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

    let store = Arc::new(RwLock::new(RedisStore::new()));
    let master_id = random_sha1_hex();

    let role = match &args.replica {
        Some(replica_args) => {
            let address = format!("{}:{}", &replica_args[0], &replica_args[1]);
            let master_stream = TcpStream::connect(address).await?;
            let replica_stream_info = StreamInfo {
                role: StreamType::Slave,
                master_id: master_id.clone(),
                master_offset: 0,
            };

            let store_clone = Arc::clone(&store);
            tokio::spawn(async move {
                match perform_handshake(master_stream).await {
                    Ok(master_stream) => {
                        if let Err(err) = RespHandler::handle_stream(
                            master_stream,
                            &store_clone,
                            replica_stream_info.clone(),
                        )
                        .await
                        {
                            eprintln!("Error in handle_master_stream: {:?}", err);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error in perform_handshake: {:?}", err);
                    }
                }
            });

            StreamType::Slave
        }
        None => StreamType::Master,
    };

    let stream_info = StreamInfo {
        role,
        master_id: master_id.clone(),
        master_offset: 0,
    };

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .context("Failed to accept incoming connection")?;
        println!("Accepted new connection");
        let store_clone = Arc::clone(&store);
        let stream_info = stream_info.clone();

        tokio::spawn(async move {
            let _ = RespHandler::handle_stream(stream, &store_clone, stream_info).await;
        });
    }
}
