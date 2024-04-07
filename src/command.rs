use self::{echo::EchoCommand, ping::PingCommand};
use anyhow::{anyhow, Result};
use tokio::net::TcpStream;

mod echo;
mod ping;

pub struct RedisCommand {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommandRegistry {
    ping_command: PingCommand,
    echo_command: EchoCommand,
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
        }
    }

    pub async fn execute(&self, stream: &mut TcpStream, command: &RedisCommand) -> Result<()> {
        match command.name.to_lowercase().as_str() {
            "ping" => self.ping_command.execute(stream).await,
            "echo" => self.echo_command.execute(stream, command).await,
            _ => Err(anyhow!("Unknown command: {}", command.name)),
        }
    }
}
