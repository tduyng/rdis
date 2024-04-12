use self::{
    echo::EchoCommand, get::GetCommand, info::InfoCommand, ping::PingCommand, psync::PsyncCommand,
    replconf::ReplConfCommand, set::SetCommand,
};
use crate::{protocol::parser::RespValue, store::RedisStore, stream::StreamInfo};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod echo;
pub mod get;
pub mod info;
pub mod ping;
pub mod psync;
pub mod replconf;
pub mod set;

#[derive(Debug, Clone)]
pub struct RedisCommandInfo {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RedisCommand {}

impl RedisCommand {
    pub async fn execute(
        stream_info: &StreamInfo,
        cmd_info: &mut RedisCommandInfo,
        store: &Arc<Mutex<RedisStore>>,
    ) -> Result<String> {
        match cmd_info.name.to_lowercase().as_str() {
            "ping" => PingCommand::execute().await,
            "echo" => EchoCommand::execute(cmd_info).await,
            "get" => GetCommand::execute(cmd_info, store).await,
            "set" => SetCommand::execute(cmd_info, store).await,
            "info" => InfoCommand::execute(stream_info).await,
            "replconf" => ReplConfCommand::execute().await,
            "psync" => PsyncCommand::execute(stream_info).await,
            _ => Err(anyhow!("Unknown command: {}", cmd_info.name)),
        }
    }
}

impl RedisCommandInfo {
    pub fn new(name: String, args: Vec<String>) -> Self {
        RedisCommandInfo { name, args }
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
}
