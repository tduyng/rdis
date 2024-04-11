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

            if is_write_command(&cmd_info) {
                cmd_info.propagate(&mut handler, store).await?;
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

fn is_write_command(command: &RedisCommandInfo) -> bool {
    let write_commands = ["set", "del"];
    write_commands.contains(&command.name.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_write_command_write() {
        let command = RedisCommandInfo {
            name: String::from("SET"),
            args: vec![],
        };
        assert_eq!(is_write_command(&command), true);
    }

    #[test]
    fn test_is_write_command_not_write() {
        let command = RedisCommandInfo {
            name: String::from("GET"),
            args: vec![],
        };
        assert_eq!(is_write_command(&command), false);
    }

    #[test]
    fn test_is_write_command_case_insensitive() {
        let command = RedisCommandInfo {
            name: String::from("DeL"),
            args: vec![],
        };
        assert_eq!(is_write_command(&command), true);
    }
}
