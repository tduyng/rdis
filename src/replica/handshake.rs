use crate::{message::Message, stream::StreamInfo};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};

use super::{connection::ReplicaConnection, get_master_socket_addr};

pub async fn perform_handshake_to_master(stream_info: &Arc<Mutex<StreamInfo>>) -> Result<ReplicaConnection> {
    let stream_info = stream_info.lock().await;
    let socket_addr = get_master_socket_addr(&stream_info);
    if socket_addr.is_none() {
        return Err(anyhow!("invalid socket address"));
    }
    let socket_addr = socket_addr.unwrap();
    let master_stream = TcpStream::connect(socket_addr).await?;
    let mut replica_connection = ReplicaConnection::bind(master_stream);

    {
        // Send PING
        let ping = Message::encode_array_str(vec!["PING"]);
        _ = replica_connection.write_bytes(ping.as_bytes()).await;
        replica_connection.get_response().await;
    }

    {
        // Send REPLCONF listening-port
        let replcon = Message::encode_array_str(vec!["REPLCONF", "listening-port", "6380"]);
        _ = replica_connection.write_bytes(replcon.as_bytes()).await;
        replica_connection.get_response().await;
    }

    {
        // Send REPLCONF capa eof and capa psync2
        let replconf = Message::encode_array_str(vec!["REPLCONF", "capa", "eof", "capa", "psync2"]);
        _ = replica_connection.write_bytes(replconf.as_bytes()).await;
        replica_connection.get_response().await;
    }

    {
        // Send PSYNC
        let psync = Message::encode_array_str(vec!["PSYNC", "?", "-1"]);
        _ = replica_connection.write_bytes(psync.as_bytes()).await;
        replica_connection.get_response().await;
    }

    Ok(replica_connection)
}
