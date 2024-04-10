use anyhow::{Context, Result};
use clap::Parser;
use redis_starter_rust::{
    replication::{handshake::perform_hashshake, ReplicaInfo, StreamType},
    stream::RespHandler,
    utils::random_sha1_hex,
};
use tokio::net::TcpListener;

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
        Some(args) => {
            perform_hashshake(args)
                .await
                .context("Failed to perform replica handshake")?; // Do we need to return master_stream here??? And use them in handle_replica?
            StreamType::Slave
        }
    };
    let replica_info = ReplicaInfo {
        role,
        repl_id: random_sha1_hex(),
        repl_offset: 0,
    };

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .context("Failed to accept incoming connection")?;
        tokio::spawn(RespHandler::handle_stream(stream, replica_info.clone()));
    }
}
