use crate::{
    command::{RedisCommand, RedisCommandInfo},
    protocol::{parser::RespValue, rdb::Rdb},
    replica::ReplInfo,
    store::RedisStore,
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::RwLock,
};

pub struct RespHandler {
    pub buffer: BytesMut,
    pub repl_info: ReplInfo,
}

impl RespHandler {
    pub async fn new(repl_info: ReplInfo) -> Self {
        RespHandler {
            buffer: BytesMut::with_capacity(512),
            repl_info,
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
        repl_info: ReplInfo,
    ) -> Result<()> {
        let mut handler = RespHandler::new(repl_info).await;

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
                        handler.repl_info.master_id
                    ))
                    .encode();
                    stream.write_all(full_resync.as_bytes()).await?;

                    let empty_rdb = Rdb::get_empty();
                    stream.write_all(&empty_rdb).await?;

                    let mut store = store.write().await;
                    store.add_repl_streams(stream);
                    return Ok(());
                }
                _ => {
                    let response = RedisCommand::execute(&mut handler, &cmd_info, store).await?;
                    stream.write_all(response.as_bytes()).await?
                }
            }

            if cmd_info.is_write() {
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
