use anyhow::{anyhow, Result};
use tokio::net::TcpStream;

mod echo;
mod ping;

use echo::EchoCommand;
use ping::PingCommand;

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

    pub async fn execute(
        &self,
        command_name: &str,
        stream: &mut TcpStream,
        args: Vec<String>,
    ) -> Result<()> {
        match command_name {
            "ping" => self.ping_command.execute(stream, args).await,
            "echo" => self.echo_command.execute(stream, args).await,
            _ => Err(anyhow!("Unknown command: {}", command_name)),
        }
    }
}
