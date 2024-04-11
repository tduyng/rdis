use crate::{
    command::{RedisCommand, RedisCommandInfo},
    store::Database,
    protocol::parser::RespValue,
    replication::ReplicaInfo,
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

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

    pub async fn handle_stream(stream: TcpStream, replica_info: ReplicaInfo) -> Result<()> {
        println!("Accepted new connection");
        let mut handler = RespHandler::new(stream, replica_info).await;

        loop {
            let value = handler.read_value().await?;

            let response = match value {
                Some(v) => {
                    let command = parse_command(v)?;
                    if is_write_command(&command) {
                        handler.propagate_command(&command).await?;
                    }
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

    pub async fn propagate_command(&mut self, command: &RedisCommandInfo) -> Result<()> {
        let resp_array = encode_array_command(command);
        let resp_value = RespValue::Array(resp_array);
        let resp_str = resp_value.encode();
        self.stream.write_all(resp_str.as_bytes()).await?;
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

fn is_write_command(command: &RedisCommandInfo) -> bool {
    let write_commands = ["set", "del"];
    write_commands.contains(&command.name.to_lowercase().as_str())
}

fn encode_array_command(command: &RedisCommandInfo) -> Vec<RespValue> {
    let mut array_values = Vec::with_capacity(command.args.len() + 1);
    array_values.push(RespValue::BulkString(command.name.clone()));
    for arg in &command.args {
        array_values.push(RespValue::BulkString(arg.clone()));
    }
    array_values
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
