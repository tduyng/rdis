use crate::{
    command::{RedisCommand, RedisCommandInfo},
    database::Database,
    protocol::parser::RespValue,
    replication::ReplicaInfo,
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn handle_stream(stream: TcpStream, replica_info: ReplicaInfo) -> Result<()> {
    println!("Accepted new connection");

    let mut handler = RespHandler::new(stream, replica_info).await;

    loop {
        let value = handler.read_value().await?;

        let response = match value {
            Some(v) => {
                let command = parse_command(v)?;
                RedisCommand::execute(&mut handler, &command).await
            }
            None => break,
        };

        if let Err(err) = response {
            println!("Error executing command: {}", err);
        }
    }

    Ok(())
}

pub struct RespHandler {
    pub stream: TcpStream,
    pub buffer: BytesMut,
    pub database: Database,
    pub replica_info: ReplicaInfo,
}

impl RespHandler {
    pub async fn new(stream: TcpStream, replica_info: ReplicaInfo) -> Self {
        let database = Database::instance().await;
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
            database,
            replica_info,
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<RespValue>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let (v, _) = RespValue::decode(self.buffer.split())?;
        Ok(Some(v))
    }

    pub async fn write_response(&mut self, response: String) -> Result<()> {
        self.stream.write_all(response.as_bytes()).await?;
        Ok(())
    }
}

fn parse_command(value: RespValue) -> Result<RedisCommandInfo> {
    match value {
        RespValue::Array(a) => {
            if let Some(command) = a.first().and_then(|v| unpack_bulk_str(v.clone())) {
                let args: Vec<String> = a.into_iter().skip(1).filter_map(unpack_bulk_str).collect();
                Ok(RedisCommandInfo {
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

fn unpack_bulk_str(value: RespValue) -> Option<String> {
    match value {
        RespValue::BulkString(s) => Some(s),
        _ => None,
    }
}
