use crate::protocol::parser::RespValue;
use anyhow::{anyhow, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn perform_replica_handshake(args: &[String]) -> Result<()> {
    let master_host = &args[0];
    let master_port: u16 = args[1].parse()?;
    let address = format!("{}:{}", master_host, master_port);
    let mut stream = TcpStream::connect(address)
        .await
        .map_err(|e| anyhow!("Failed to connect to master: {}", e))?;
    println!("Replica running on {}:{}", master_host, master_port);

    // Send PING command
    let ping_command = RespValue::encode_array_str(vec!["PING"]);
    stream.write_all(ping_command.as_bytes()).await?;

    // Receive response to PING
    let mut buf = [0; 512];
    let _ = stream.read(&mut buf).await?;

    // Send REPLCONF listening-port <PORT>
    let replconf_command = RespValue::encode_array_str(vec!["REPLCONF", "listening-port", "6380"]);
    stream.write_all(replconf_command.as_bytes()).await?;

    // Receive response to REPLCONF listening-port
    let _ = stream.read(&mut buf).await?;

    // Send REPLCONF capa eof capa psync2
    let replconf_command =
        RespValue::encode_array_str(vec!["REPLCONF", "capa", "eof", "capa", "psync2"]);
    stream.write_all(replconf_command.as_bytes()).await?;

    // Receive response to REPLCONF capa eof capa psync2
    let _ = stream.read(&mut buf).await?;

    // Send PSYNC ? -1
    let psync_command = RespValue::encode_array_str(vec!["PSYNC", "?", "-1"]);
    stream.write_all(psync_command.as_bytes()).await?;

    Ok(())
}
