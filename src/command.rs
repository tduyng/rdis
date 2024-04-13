use crate::{protocol::parser::RespValue, replica::ReplicaCommand, store::Entry};
use anyhow::Result;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Echo(String),
    Ping,
    Quit,
    Set(String, Entry),
    Get(String),
    Info,
    Replconf(Vec<String>),
    Psync,
}

impl Command {
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

impl CommandInfo {
    pub fn new(name: String, args: Vec<String>) -> Self {
        CommandInfo { name, args }
    }

    pub fn to_command(&self) -> Option<Command> {
        let args_clone = self.args.clone();
        match self.name.to_lowercase().as_str() {
            "ping" => Some(Command::Ping),
            "echo" => Some(Command::Echo(args_clone.join(" "))),
            "get" => Some(Command::Get(args_clone[0].clone())),
            "set" => {
                let (key, value) = self.get_key_value().unwrap();
                let expiry = self.get_expiry();
                let entry = Entry::new(value, expiry);

                Some(Command::Set(key, entry))
            }
            "info" => Some(Command::Info),
            "replconf" => Some(Command::Replconf(args_clone)),
            "psync" => Some(Command::Psync),
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
