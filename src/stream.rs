use crate::{
    command::{CommandRegistry, RedisCommand},
    database::Database,
    protocol::parser::{parse_message, RedisValue},
    replication::ReplicaMode,
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn handle_stream(
    stream: TcpStream,
    redis_command: CommandRegistry,
    replica_mode: ReplicaMode,
) -> Result<()> {
    println!("Accepted new connection");

    let mut handler = ResponseHandler::new(stream, replica_mode);

    loop {
        let value = handler.read_value().await?;

        let response = match value {
            Some(v) => {
                let command = parse_command(v)?;
                redis_command.execute(&mut handler, &command).await
            }
            None => break,
        };

        if let Err(err) = response {
            println!("Error executing command: {}", err);
        }
    }

    Ok(())
}

pub struct ResponseHandler {
    pub stream: TcpStream,
    pub buffer: BytesMut,
    pub database: Database,
    pub replica_mode: ReplicaMode,
}

impl ResponseHandler {
    fn new(stream: TcpStream, replica_mode: ReplicaMode) -> Self {
        ResponseHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
            database: Database::new(),
            replica_mode,
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<RedisValue>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let (v, _) = parse_message(self.buffer.split())?;
        Ok(Some(v))
    }

    pub async fn write_response(&mut self, response: String) -> Result<()> {
        self.stream.write_all(response.as_bytes()).await?;
        Ok(())
    }
}

fn parse_command(value: RedisValue) -> Result<RedisCommand> {
    match value {
        RedisValue::Array(a) => {
            if let Some(command) = a.first().and_then(|v| unpack_bulk_str(v.clone())) {
                let args: Vec<String> = a.into_iter().skip(1).filter_map(unpack_bulk_str).collect();
                Ok(RedisCommand {
                    name: command,
                    args,
                })
            } else {
                Err(anyhow!("Invalid command format"))
            }
        }
        _ => Err(anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: RedisValue) -> Option<String> {
    match value {
        RedisValue::BulkString(s) => Some(s),
        _ => None,
    }
}
