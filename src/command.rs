use crate::{
    protocol::parser::RespValue,
    replica::ReplicaCommand,
    store::{Entry, RedisStore},
    stream::StreamInfo,
};
use anyhow::{anyhow, Result};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RedisCommandInfo {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RedisCommand {
    Echo(String),
    Ping,
    Quit,
    Set(String, Entry),
    Get(String),
    Info,
    Replconf,
    Psync,
}

impl RedisCommand {
    pub async fn execute(
        stream_info: &StreamInfo,
        cmd_info: &mut RedisCommandInfo,
        store: &Arc<Mutex<RedisStore>>,
    ) -> Result<String> {
        match cmd_info.to_command() {
            Some(command) => match command {
                Self::Ping => Ok(RespValue::SimpleString("PONG".to_string()).encode()),
                Self::Echo(message) => Ok(RespValue::SimpleString(message).encode()),
                Self::Get(key) => {
                    if let Some(entry) = store.lock().await.get(key) {
                        let response = format!("${}\r\n{}\r\n", entry.value.len(), entry.value);
                        Ok(response)
                    } else {
                        Ok("$-1\r\n".to_string())
                    }
                }
                Self::Set(key, entry) => {
                    store.lock().await.set(key, entry);
                    Ok(RespValue::SimpleString("OK".to_string()).encode())
                }
                Self::Info => {
                    let response = format!(
                        "# Replication\n\
                        role:{}\n\
                        connected_clients:{}\n\
                        master_replid:{}\n\
                        master_repl_offset:{}\n\
                        ",
                        stream_info.role,
                        stream_info.connected_clients,
                        stream_info.id,
                        stream_info.offset
                    );
                    Ok(RespValue::BulkString(response).encode())
                }
                Self::Replconf => Ok(RespValue::SimpleString("OK".to_string()).encode()),
                Self::Psync => Ok(RespValue::SimpleString(format!(
                    "FULLRESYNC {} 0",
                    stream_info.id
                ))
                .encode()),
                _ => Err(anyhow!("Unknown command")),
            },
            None => Err(anyhow!("Invalid command info")),
        }
    }

    pub fn to_replica_command(&self) -> Option<ReplicaCommand> {
        match self {
            Self::Set(key, entry) => {
                let message = if entry.expiry_at.is_some() {
                    RespValue::Array(vec![
                        RespValue::BulkString("set".to_string()),
                        RespValue::BulkString(key.clone()),
                        RespValue::BulkString(entry.value.clone()),
                        RespValue::BulkString("px".to_string()),
                        RespValue::BulkString(entry.expiry_time.unwrap().as_millis().to_string()),
                    ])
                } else {
                    RespValue::Array(vec![
                        RespValue::BulkString("set".to_string()),
                        RespValue::BulkString(key.clone()),
                        RespValue::BulkString(entry.value.clone()),
                    ])
                };

                Some(ReplicaCommand { message })
            }
            _ => None,
        }
    }
}

impl RedisCommandInfo {
    pub fn new(name: String, args: Vec<String>) -> Self {
        RedisCommandInfo { name, args }
    }

    pub fn to_command(&self) -> Option<RedisCommand> {
        match self.name.to_lowercase().as_str() {
            "ping" => Some(RedisCommand::Ping),
            "echo" => Some(RedisCommand::Echo(self.args.join(" "))),
            "get" => Some(RedisCommand::Get(self.args[0].clone())),
            "set" => {
                let (key, value) = self.get_key_value().unwrap();
                let expiry = self.get_expiry();
                let entry = Entry::new(value, expiry);

                Some(RedisCommand::Set(key, entry))
            }
            "info" => Some(RedisCommand::Info),
            "replconf" => Some(RedisCommand::Replconf),
            "psync" => Some(RedisCommand::Psync),
            _ => None,
        }
    }

    pub fn encode(&self) -> String {
        let mut array_values = Vec::with_capacity(self.args.len() + 1);
        array_values.push(RespValue::BulkString(self.name.clone()));
        for arg in &self.args {
            array_values.push(RespValue::BulkString(arg.clone()));
        }
        RespValue::Array(array_values).encode()
    }

    pub fn is_write(&self) -> bool {
        let write_commands = ["set", "del"];
        write_commands.contains(&self.name.to_lowercase().as_str())
    }

    fn get_key_value(&self) -> Result<(String, String)> {
        if self.args.len() < 2 {
            return Err(anyhow::anyhow!(
                "SET command requires exactly two arguments"
            ));
        }
        let key = self.args[0].clone();
        let value = self.args[1].clone();

        Ok((key, value))
    }

    fn get_expiry(&self) -> Option<Duration> {
        if self.args.len() < 4 {
            return None;
        }
        if let Some(tag) = self.args.get(2) {
            if tag != "px" {
                return None;
            }
        }
        if let Some(duration) = self.args.get(3) {
            let duration_time = duration.parse::<u64>().unwrap_or_default();
            return Some(Duration::from_millis(duration_time));
        }
        None
    }
}
