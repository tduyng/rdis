use crate::{command::CommandRegistry, protocol::parser::parse_command};
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
        let command = parse_command(&buffer[..bytes_read])?;

        redis_command.execute(&mut stream, &command).await?;
    }
    Ok(())
}
