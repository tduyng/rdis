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
    stream.write_all(b"*1\r\n$4\r\nping\r\n").await?;

    // Receive response to PING
    let mut buf = Vec::from([0; 124]);
    let _ = stream.read(&mut buf).await;

    // Send REPLCONF listening-port <PORT>
    stream
        .write_all(b"*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n")
        .await?;

    // Receive response to REPLCONF listening-port
    let _ = stream.read(&mut buf).await;

    // Send REPLCONF capa eof capa psync2
    stream
        .write_all(
            b"*5\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$3\r\neof\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n",
        )
        .await?;

    // Receive response to REPLCONF capa eof capa psync2
    let _ = stream.read(&mut buf).await;

    // Send PSYNC ? -1
    stream
        .write_all(b"*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n")
        .await?;

    Ok(())
}
