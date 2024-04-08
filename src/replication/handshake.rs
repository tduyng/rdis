use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn perform_handshake(args: &[String]) -> Result<()> {
    let master_host = &args[0];
    let master_port: u16 = args[1].parse()?;
    let mut stream = TcpStream::connect(format!("{}:{}", master_host, master_port)).await?;
    let ping_command = format!("*1\r\n$4\r\nping\r\n");

    stream.write_all(ping_command.as_bytes()).await?;
    Ok(())
}
