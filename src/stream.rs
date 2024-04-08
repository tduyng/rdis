use crate::{command::CommandRegistry, database::Database, protocol::parser::parse_command};
use anyhow::Result;
use bytes::BytesMut;
use tokio::{io::AsyncReadExt, net::TcpStream};

pub async fn handle_stream(mut stream: TcpStream, redis_command: CommandRegistry) -> Result<()> {
    println!("Accepted new connection");

    let mut database = Database::new();
    let mut buffer = BytesMut::with_capacity(512);

    loop {
        buffer.clear();
        let bytes_read = stream.read_buf(&mut buffer).await?;

        if bytes_read == 0 {
            break;
        }

        let command = parse_command(&mut buffer)?;

        redis_command
            .execute(&mut stream, &mut database, &command)
            .await?;
    }
    Ok(())
}
