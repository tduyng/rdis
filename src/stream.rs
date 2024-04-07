use crate::command::CommandRegistry;
use anyhow::Result;
use tokio::{io::AsyncReadExt, net::TcpStream};

pub async fn handle_stream(mut stream: TcpStream, redis_command: CommandRegistry) -> Result<()> {
    println!("Accepted new connection");

    let mut buffer = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        let raw_request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let parts: Vec<&str> = raw_request.split_whitespace().map(|s| s.trim()).collect();
        if parts.is_empty() {
            continue;
        }
        let command_name = parts[0].to_lowercase();
        let args = parts[1..].iter().map(|&s| s.to_string()).collect();

        redis_command
            .execute(&command_name, &mut stream, args)
            .await?;
    }
    Ok(())
}
