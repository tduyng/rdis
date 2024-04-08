use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use anyhow::Result;
use clap::Parser;
use redis_starter_rust::{
    command::CommandRegistry, replication::ReplicaMode, stream::handle_stream,
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
            assert_eq!(args.len(), 2);
            let addr = if args.first().unwrap() == "localhost" {
                IpAddr::from_str("127.0.0.1").unwrap()
            } else {
                IpAddr::from_str(args.first().unwrap()).unwrap()
            };
            let port: u16 = args.get(1).unwrap().clone().parse().unwrap();
            ReplicaMode::Slave(SocketAddr::new(addr, port))
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
