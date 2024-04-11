use self::{
    echo::EchoCommand, get::GetCommand, info::InfoCommand, ping::PingCommand,
    replconf::ReplConfCommand, set::SetCommand,
};
use crate::{protocol::parser::RespValue, store::RedisStore, stream::RespHandler};
use anyhow::{anyhow, Result};
use tokio::{io::AsyncWriteExt, sync::RwLock};

pub mod echo;
pub mod get;
pub mod info;
pub mod ping;
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
        handler: &mut RespHandler,
        cmd_info: &RedisCommandInfo,
        store: &RwLock<RedisStore>,
    ) -> Result<String> {
        match cmd_info.name.to_lowercase().as_str() {
            "ping" => PingCommand::execute().await,
            "echo" => EchoCommand::execute(cmd_info).await,
            "get" => GetCommand::execute(cmd_info, store).await,
            "set" => SetCommand::execute(cmd_info, store).await,
            "info" => InfoCommand::execute(handler).await,
            "replconf" => ReplConfCommand::execute().await,
            _ => Err(anyhow!("Unknown command: {}", cmd_info.name)),
        }
    }
}

impl RedisCommandInfo {
    pub fn new(name: String, args: Vec<String>) -> Self {
        RedisCommandInfo { name, args }
    }

    pub fn encode(&self) -> Vec<RespValue> {
        let mut array_values = Vec::with_capacity(self.args.len() + 1);
        array_values.push(RespValue::BulkString(self.name.clone()));
        for arg in &self.args {
            array_values.push(RespValue::BulkString(arg.clone()));
        }
        array_values
    }

    pub async fn propagate(&mut self, store: &RwLock<RedisStore>) -> Result<()> {
        let encoded_command = RespValue::Array(self.encode()).encode();
        let mut store_guard = store.write().await;
        for stream in store_guard.repl_streams.iter_mut() {
            stream.write_all(encoded_command.as_bytes()).await?;
            SetCommand::execute(self, store).await?;
        }
        Ok(())
    }

    pub fn is_write(&self) -> bool {
        let write_commands = ["set", "del"];
        write_commands.contains(&self.name.to_lowercase().as_str())
    }
}
