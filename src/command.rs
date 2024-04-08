use crate::stream::ResponseHandler;

use self::{echo::EchoCommand, get::GetCommand, ping::PingCommand, set::SetCommand};
use anyhow::{anyhow, Result};

mod echo;
mod get;
mod ping;
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
            _ => Err(anyhow!("Unknown command: {}", command.name)),
        }
    }
}
