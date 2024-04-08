use anyhow::{anyhow, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn perform_handshake(args: &[String]) -> Result<()> {
    let master_host = &args[0];
    let master_port: u16 = args[1].parse()?;
    let mut stream = TcpStream::connect(format!("{}:{}", master_host, master_port))
        .await
        .map_err(|e| anyhow!("Failed to connect to master: {}", e))?;

    send_ping(&mut stream).await?;
    receive_ok_response(&mut stream).await?;

    send_replconf_listen_port(&mut stream, args).await?;
    receive_ok_response(&mut stream).await?;

    send_replconf_capa_psync2(&mut stream).await?;
    receive_ok_response(&mut stream).await?;

    Ok(())
}

async fn send_ping(stream: &mut TcpStream) -> Result<()> {
    let ping_command = "*1\r\n$4\r\nping\r\n".to_string();
    stream.write_all(ping_command.as_bytes()).await?;
    Ok(())
}

async fn send_replconf_listen_port(stream: &mut TcpStream, args: &[String]) -> Result<()> {
    let replconf_listen_port = format!(
        "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n",
        args[1].len(),
        args[1]
    );

    stream.write_all(replconf_listen_port.as_bytes()).await?;
    Ok(())
}

async fn send_replconf_capa_psync2(stream: &mut TcpStream) -> Result<()> {
    let replconf_capa_psync2 = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n".to_string();
    stream.write_all(replconf_capa_psync2.as_bytes()).await?;
    Ok(())
}

async fn receive_ok_response(stream: &mut TcpStream) -> Result<()> {
    let mut buffer = [0; 32];
    stream.read_exact(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..]);
    if !response.starts_with("+OK\r\n") {
        return Err(anyhow!("Unexpected response: {}", response));
    }
    Ok(())
}
