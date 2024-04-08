use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use redis_starter_rust::{command::CommandRegistry, replication::ReplicaMode, stream::handle_stream};
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

    let replica_mode = if let Some(replicaof) = args.replicaof {
        let master_host = replicaof[0].clone();
        let master_port: u16 = replicaof[1].parse().unwrap();
        let master_addr = SocketAddr::new(master_host.parse().unwrap(), master_port);
        ReplicaMode::Slave(master_addr)
    } else {
        ReplicaMode::Master
    };

    let registry = CommandRegistry::new();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_stream(stream, registry.clone(), replica_mode.clone()));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
