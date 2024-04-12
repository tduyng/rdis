use anyhow::{Context, Result};
use clap::Parser;
use redis_starter_rust::{
    handler::Handler,
    replica::{handshake::perform_handshake_to_master, should_replicate},
    store::RedisStore,
    stream::StreamInfo,
};
use std::{
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    sync::Arc,
};
use tokio::{
    net::TcpListener,
    sync::{Mutex, RwLock},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value = "127.0.0.1")]
    #[clap(short, long)]
    address: IpAddr,

    #[clap(short, long, default_value = "6379")]
    port: u16,

    #[clap(short, long = "replicaof", value_names = &["MASTER_HOST", "MASTER_PORT"], num_args = 2)]
    replica: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let repl_addr = match &args.replica {
        Some(replica_args) => {
            let server = format!("{}:{}", &replica_args[0], &replica_args[1].parse::<u16>()?);

            if let Ok(socket) = server.to_socket_addrs() {
                let server: Vec<_> = socket.collect();
                let addr = server.first().expect("No valid address found");
                Some(*addr)
            } else {
                None
            }
        }
        None => None,
    };
    let stream_info = Arc::new(Mutex::new(StreamInfo::new(repl_addr)));

    {
        let info = stream_info.lock().await;
        if should_replicate(&info) {
            perform_handshake_to_master(&info).await?;
        }
    }

    let store = Arc::new(RwLock::new(RedisStore::new()));
    let socket_address = SocketAddr::new(args.address, args.port);
    let listener = TcpListener::bind(socket_address)
        .await
        .context("Failed to bind to address")?;
    println!("Server listening on {}", socket_address);

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
