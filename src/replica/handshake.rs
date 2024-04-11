use crate::protocol::parser::RespValue;
use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn perform_handshake(mut master_stream: TcpStream) -> Result<TcpStream> {
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
