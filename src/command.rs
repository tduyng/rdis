use crate::stream::RespHandler;

use self::{
    echo::EchoCommand, get::GetCommand, info::InfoCommand, ping::PingCommand, psync::PsyncCommand,
    replconf::ReplConfCommand, set::SetCommand,
};
use anyhow::{anyhow, Result};

mod echo;
mod get;
mod info;
mod ping;
mod psync;
mod replconf;
mod set;

pub struct RedisCommandInfo {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RedisCommand {}

impl RedisCommand {
    pub async fn execute(handler: &mut RespHandler, command: &RedisCommandInfo) -> Result<()> {
        match command.name.to_lowercase().as_str() {
            "ping" => PingCommand::execute(handler).await,
            "echo" => EchoCommand::execute(handler, command).await,
            "get" => GetCommand::execute(handler, command).await,
            "set" => SetCommand::execute(handler, command).await,
            "info" => InfoCommand::execute(handler).await,
            "replconf" => ReplConfCommand::execute(handler).await,
            "psync" => PsyncCommand::execute(handler).await,
            _ => Err(anyhow!("Unknown command: {}", command.name)),
        }
    }
}
