use crate::{message::Message, replica::ReplicaCommand, store::Entry};
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
    Wait(u64),
    Config(String, String),
}

impl Command {
    pub fn to_replica_command(cmd: &Command) -> Option<ReplicaCommand> {
        match cmd {
            Self::Set(key, entry) => {
                let message = if entry.expiry_at.is_some() {
                    Message::Array(vec![
                        Message::Bulk("set".to_string()),
                        Message::Bulk(key.clone()),
                        Message::Bulk(entry.value.clone()),
                        Message::Bulk("px".to_string()),
                        Message::Bulk(entry.expiry_time.unwrap().as_millis().to_string()),
                    ])
                } else {
                    Message::Array(vec![
                        Message::Bulk("set".to_string()),
                        Message::Bulk(key.clone()),
                        Message::Bulk(entry.value.clone()),
                    ])
                };

                Some(ReplicaCommand { message, timeout: None })
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
            "wait" => {
                let timeout = self.args[1].parse::<u64>().unwrap(); // first args is number of replicas
                Some(Command::Wait(timeout))
            }
            "config" => {
                let action = self.args[0].clone();
                let key = self.args[1].clone();
                Some(Command::Config(action, key))
            }
            _ => None,
        }
    }

    pub fn encode(&self) -> String {
        let mut array_values = Vec::with_capacity(self.args.len() + 1);
        array_values.push(Message::Bulk(self.name.clone()));
        for arg in &self.args {
            array_values.push(Message::Bulk(arg.clone()));
        }
        Message::Array(array_values).encode()
    }

    pub fn is_write(&self) -> bool {
        let write_commands = ["set", "del"];
        write_commands.contains(&self.name.to_lowercase().as_str())
    }

    fn get_key_value(&self) -> Result<(String, String)> {
        if self.args.len() < 2 {
            return Err(anyhow::anyhow!("SET command requires exactly two arguments"));
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
