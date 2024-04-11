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
                _ => RedisCommand::execute(stream, &mut handler, &cmd_info, store).await?,
            }

            if is_write_command(&cmd_info) {
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

    #[tokio::test]
    async fn test_parse_command() {
        let input = RespValue::Array(vec![
            RespValue::BulkString("SET".to_string()),
            RespValue::BulkString("key".to_string()),
            RespValue::BulkString("value".to_string()),
        ]);
        let result = parse_command(input);
        assert!(result.is_ok());
        let command_info = result.unwrap();
        assert_eq!(command_info.name, "SET");
        assert_eq!(
            command_info.args,
            vec!["key".to_string(), "value".to_string()]
        );
    }

    #[test]
    fn test_unpack_bulk_str() {
        let input = RespValue::BulkString("value".to_string());
        let result = unpack_bulk_str(input);
        assert_eq!(result, Some("value".to_string()));

        let invalid_input = RespValue::SimpleString("OK".to_string());
        let result = unpack_bulk_str(invalid_input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_encode_array_command() {
        let command_info = RedisCommandInfo {
            name: "SET".to_string(),
            args: vec!["key".to_string(), "value".to_string()],
        };
        let result = encode_array_command(&command_info);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], RespValue::BulkString("SET".to_string()));
        assert_eq!(result[1], RespValue::BulkString("key".to_string()));
        assert_eq!(result[2], RespValue::BulkString("value".to_string()));
    }
}
