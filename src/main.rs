use anyhow::Result;
use clap::Parser;
use redis_starter_rust::{
    replication::{handshake::perform_replica_handshake, ReplicaInfo, StreamType},
    stream::handle_stream,
    utils::random_sha1_hex,
};
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "6379")]
    port: u16,
    #[arg(short,long = "replicaof", value_names = &["MASTER_HOST", "MASTER_PORT"], num_args = 2)]
    replicaof: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    println!("Server listening on 127.0.0.1:{}", args.port);

    let stream_type = match &args.replicaof {
        None => StreamType::Master,
        Some(args) => {
            perform_replica_handshake(args).await?;
            StreamType::Slave
        }
    };
    let replica_info = ReplicaInfo {
        stream_type,
        master_replid: random_sha1_hex(),
        master_repl_offset: 0,
    };

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_stream(stream, replica_info.clone()));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
