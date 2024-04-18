use anyhow::{Context, Result};
use clap::Parser;
use redis_starter_rust::{
    args::CliArgs,
    connection::Connection,
    handler::Handler,
    protocol::rdb::Rdb,
    replica::{handler::ReplicaHandler, handshake::perform_handshake_to_master, should_replicate},
    store::Store,
    stream::StreamInfo,
};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    let stream_info = Arc::new(StreamInfo::new(&args));
    let store = Arc::new(Mutex::new(Store::new()));
    if let Some(dir) = args.dir {
        stream_info.config.lock().await.dir = Some(dir);
    }
    if let Some(dbfilename) = args.dbfilename {
        stream_info.config.lock().await.dbfilename = Some(dbfilename);
    }

    {
        let rdb_data = Rdb::read_file(&stream_info).await;
        if let Some(data) = rdb_data {
            store.lock().await.import_rdb(&data);
        }
    }

    if should_replicate(&stream_info).await {
        let info = stream_info.clone();
        let store = store.clone();
        tokio::spawn(async move {
            let mut replica_connection = perform_handshake_to_master(&info)
                .await
                .expect("Failed the handshake with the master");
            replica_connection.get_rdb().await;
            _ = ReplicaHandler::handle_replica(replica_connection, store).await;
        });
    }

    let socket_addr = stream_info.socket_addr;
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
        let connection = Connection::bind(stream);
        println!("Accepted new connection");

        tokio::spawn(async move {
            let _ = Handler::handle_stream(connection, store, stream_info).await;
        });
    }
}
