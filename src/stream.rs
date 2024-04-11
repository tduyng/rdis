use crate::{
    command::{RedisCommand, RedisCommandInfo},
    protocol::{parser::RespValue, rdb::Rdb},
    store::RedisStore,
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::RwLock,
};

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Master,
    Slave,
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub role: StreamType,
    pub master_id: String,
    pub master_offset: u16,
}

pub struct RespHandler {
    pub buffer: BytesMut,
    pub stream_info: StreamInfo,
    pub is_master: bool,
}

impl RespHandler {
    pub async fn new(stream_info: StreamInfo) -> Self {
        let role = stream_info.role.clone();
        RespHandler {
            buffer: BytesMut::with_capacity(512),
            stream_info,
            is_master: role == StreamType::Master,
        }
    }

    pub async fn parse_command(&mut self) -> Result<RedisCommandInfo> {
        let (value, _) = RespValue::decode(self.buffer.split())?;
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
            _ => Err(anyhow!("Unexpected command format")),
        }
    }

    pub async fn handle_stream(
        mut stream: TcpStream,
        store: &RwLock<RedisStore>,
        stream_info: StreamInfo,
    ) -> Result<()> {
        let mut handler = RespHandler::new(stream_info).await;

        loop {
            let buffer_read = stream.read_buf(&mut handler.buffer).await?;
            if buffer_read == 0 {
                return Ok(());
            }

            let mut cmd_info = handler.parse_command().await?;

            match cmd_info.name.to_lowercase().as_str() {
                "psync" => {
                    let full_resync = RespValue::SimpleString(format!(
                        "FULLRESYNC {} 0",
                        handler.stream_info.master_id
                    ))
                    .encode();
                    stream.write_all(full_resync.as_bytes()).await?;

                    if handler.is_master {
                        let empty_rdb = Rdb::get_empty();
                        stream.write_all(&empty_rdb).await?;

                        let mut store = store.write().await;
                        store.add_repl_streams(stream);
                    }

                    return Ok(());
                }
                _ => {
                    let response = RedisCommand::execute(&mut handler, &cmd_info, store).await?;

                    // Do not send any response back to the master
                    if handler.is_master || !cmd_info.is_write() {
                        stream.write_all(response.as_bytes()).await?;
                    }
                }
            }

            if handler.is_master && cmd_info.is_write() {
                cmd_info.propagate(store).await?;
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
