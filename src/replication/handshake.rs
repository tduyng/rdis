use crate::protocol::parser::RedisValue;
use anyhow::{anyhow, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn perform_replica_handshake(args: &[String]) -> Result<()> {
    let master_host = &args[0];
    let master_port: u16 = args[1].parse()?;
    let mut stream = TcpStream::connect(format!("{}:{}", master_host, master_port))
        .await
        .map_err(|e| anyhow!("Failed to connect to master: {}", e))?;

    // Send PING command
    let ping_command = RedisValue::array_string(vec!["PING"]);
    stream.write_all(ping_command.as_bytes()).await?;

    // Receive response to PING
    let mut buf = [0; 512];
    let _ = stream.read(&mut buf).await?;

    // Send REPLCONF listening-port <PORT>
    let replconf_command = RedisValue::array_string(vec!["REPLCONF", "listening-port", "6380"]);
    stream.write_all(replconf_command.as_bytes()).await?;

    // Receive response to REPLCONF listening-port
    let _ = stream.read(&mut buf).await?;

    // Send REPLCONF capa eof capa psync2
    let replconf_command =
        RedisValue::array_string(vec!["REPLCONF", "capa", "eof", "capa", "psync2"]);
    stream.write_all(replconf_command.as_bytes()).await?;

    // Receive response to REPLCONF capa eof capa psync2
    let _ = stream.read(&mut buf).await?;

    // Send PSYNC ? -1
    let psync_command = RedisValue::array_string(vec!["PSYNC", "?", "-1"]);
    stream.write_all(psync_command.as_bytes()).await?;

    Ok(())
}
