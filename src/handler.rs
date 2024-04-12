use crate::{
    command::{RedisCommand, RedisCommandInfo},
    protocol::{parser::RespValue, rdb::Rdb},
    store::RedisStore,
    stream::StreamInfo,
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub struct Handler {}
impl Handler {
    pub async fn parse_command(stream: &mut TcpStream) -> Result<RedisCommandInfo> {
        let mut buffer = BytesMut::with_capacity(512);
        let bytes_to_read = stream.read_buf(&mut buffer).await?;
        if bytes_to_read == 0 {
            return Err(anyhow!("Empty buffer!"));
        };
        let (value, _) = RespValue::decode(buffer)?;
        match value {
            RespValue::Array(a) => {
                if let Some(name) = a.first().and_then(|v| unpack_bulk_str(v.clone())) {
                    let args: Vec<String> =
                        a.into_iter().skip(1).filter_map(unpack_bulk_str).collect();
                    Ok(RedisCommandInfo::new(name, args))
                } else {
                    Err(anyhow!("Invalid command format"))
                }
            }
            _ => Err(anyhow!("Unexpected command format: {:?}", value)),
        }
    }

    pub async fn handle_stream(
        mut stream: TcpStream,
        store: Arc<Mutex<RedisStore>>,
        stream_info: Arc<Mutex<StreamInfo>>,
    ) -> Result<()> {
        loop {
            let mut cmd_info = Self::parse_command(&mut stream).await?;
            let stream_info = stream_info.lock().await;

            match cmd_info.name.to_lowercase().as_str() {
                "psync" => {
                    let full_resync =
                        RespValue::SimpleString(format!("FULLRESYNC {} 0", stream_info.id))
                            .encode();
                    stream.write_all(full_resync.as_bytes()).await?;

                    let empty_rdb = Rdb::get_empty();
                    stream.write_all(&empty_rdb).await?;

                    return Ok(());
                }
                _ => {
                    let response =
                        RedisCommand::execute(&stream_info, &mut cmd_info, &store).await?;
                    stream.write_all(response.as_bytes()).await?;
                }
            }
        }
    }
}

fn unpack_bulk_str(value: RespValue) -> Option<String> {
    match value {
        RespValue::BulkString(s) => Some(s),
        _ => None,
    }
}
