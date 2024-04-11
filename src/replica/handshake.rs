use crate::{
    protocol::parser::RespValue,
    store::RedisStore,
    stream::{RespHandler, StreamInfo},
};
use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::RwLock,
};

pub async fn perform_handshake(
    mut master_stream: TcpStream,
    store: &RwLock<RedisStore>,
    stream_info: StreamInfo,
) -> Result<()> {
    // Send PING
    let ping_command = RespValue::encode_array_str(vec!["PING"]);
    master_stream.write_all(ping_command.as_bytes()).await?;

    // Read and ignore the response (PONG)
    let mut buf = [0; 512];
    let _ = master_stream.read(&mut buf).await?;

    // Send REPLCONF listening-port
    let replconf_command = RespValue::encode_array_str(vec!["REPLCONF", "listening-port", "6380"]);
    master_stream.write_all(replconf_command.as_bytes()).await?;

    // Read and ignore the response (OK)
    let _ = master_stream.read(&mut buf).await?;

    // Send REPLCONF capa eof and capa psync2
    let replconf_command =
        RespValue::encode_array_str(vec!["REPLCONF", "capa", "eof", "capa", "psync2"]);
    master_stream.write_all(replconf_command.as_bytes()).await?;

    // Read and ignore the response (OK)
    let _ = master_stream.read(&mut buf).await?;

    // Send PSYNC
    let psync_command = RespValue::encode_array_str(vec!["PSYNC", "?", "-1"]);
    master_stream.write_all(psync_command.as_bytes()).await?;

    // Read and ignore the response (the RDB file)
    let _ = master_stream.read(&mut buf).await?;

    let _ = master_stream.readable().await;
    // Once the handshake is done, start handling the replica state
    RespHandler::handle_replica_stream(master_stream, store, stream_info).await?;

    Ok(())
}
