use anyhow::Result;
use clap::Parser;
use redis_starter_rust::{
    command::CommandRegistry,
    replication::{handshake::perform_replica_handshake, ReplicaMode},
    stream::handle_stream,
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

    let mode = match &args.replicaof {
        None => ReplicaMode::Master,
        Some(args) => {
            perform_replica_handshake(args).await?;
            ReplicaMode::Slave
        }
    };

    let registry = CommandRegistry::new();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_stream(stream, registry.clone(), mode.clone()));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
