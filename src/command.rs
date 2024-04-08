use self::{echo::Echo, ping::Ping};
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
    ping: Ping,
    echo: Echo,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            ping: Ping,
            echo: Echo,
        }
    }

    pub async fn execute(&self, stream: &mut TcpStream, command: &RedisCommand) -> Result<()> {
        match command.name.to_lowercase().as_str() {
            "ping" => self.ping.execute(stream).await,
            "echo" => self.echo.execute(stream, command).await,
            _ => Err(anyhow!("Unknown command: {}", command.name)),
        }
    }
}
