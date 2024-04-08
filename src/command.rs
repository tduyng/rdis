use crate::stream::ResponseHandler;

use self::{
    echo::EchoCommand, get::GetCommand, info::InfoCommand, ping::PingCommand,
    replconf::ReplConfCommand, set::SetCommand,
};
use anyhow::{anyhow, Result};

mod echo;
mod get;
mod info;
mod ping;
mod replconf;
mod set;

pub struct RedisCommand {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommandRegistry {
    ping_command: PingCommand,
    echo_command: EchoCommand,
    get_command: GetCommand,
    set_command: SetCommand,
    info_command: InfoCommand,
    replconf_command: ReplConfCommand,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            ping_command: PingCommand,
            echo_command: EchoCommand,
            get_command: GetCommand,
            set_command: SetCommand,
            info_command: InfoCommand,
            replconf_command: ReplConfCommand,
        }
    }

    pub async fn execute(
        &self,
        handler: &mut ResponseHandler,
        command: &RedisCommand,
    ) -> Result<()> {
        match command.name.to_lowercase().as_str() {
            "ping" => self.ping_command.execute(handler).await,
            "echo" => self.echo_command.execute(handler, command).await,
            "get" => self.get_command.execute(handler, command).await,
            "set" => self.set_command.execute(handler, command).await,
            "info" => self.info_command.execute(handler).await,
            "replconf" => self.replconf_command.execute(handler).await,
            _ => Err(anyhow!("Unknown command: {}", command.name)),
        }
    }
}
