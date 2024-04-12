use crate::{protocol::parser::RespValue, stream::StreamInfo};
use anyhow::{anyhow, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use super::get_master_socket_addr;

pub async fn perform_handshake_to_master(stream_info: &StreamInfo) -> Result<TcpStream> {
    let socket_addr = get_master_socket_addr(stream_info);
    if socket_addr.is_none() {
        return Err(anyhow!("invalid socket address"));
    }
    let socket_addr = socket_addr.unwrap();
    let mut master_stream = TcpStream::connect(socket_addr).await?;

    // Send PING
    let ping_command = RespValue::encode_array_str(vec!["PING"]);
    master_stream.write_all(ping_command.as_bytes()).await?;
    let mut buf = [0; 512];
    let _ = master_stream.read(&mut buf).await?;

    // Send REPLCONF listening-port
    let replconf_command = RespValue::encode_array_str(vec!["REPLCONF", "listening-port", "6380"]);
    master_stream.write_all(replconf_command.as_bytes()).await?;
    let _ = master_stream.read(&mut buf).await?;

    // Send REPLCONF capa eof and capa psync2
    let replconf_command =
        RespValue::encode_array_str(vec!["REPLCONF", "capa", "eof", "capa", "psync2"]);
    master_stream.write_all(replconf_command.as_bytes()).await?;
    let _ = master_stream.read(&mut buf).await?;

    // Send PSYNC
    let psync_command = RespValue::encode_array_str(vec!["PSYNC", "?", "-1"]);
    master_stream.write_all(psync_command.as_bytes()).await?;
    let _ = master_stream.read(&mut buf).await?;

    Ok(master_stream)
}
