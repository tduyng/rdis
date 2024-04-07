use anyhow::{anyhow, Result};
use tokio::net::TcpStream;

mod echo;
mod ping;

use echo::EchoCommand;
use ping::PingCommand;
use crate::stream::RedisRequest;

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
        stream: &mut TcpStream,
        request: &RedisRequest
    ) -> Result<()> {
        match request.command_name.as_str() {
            "ping" => self.ping_command.execute(stream).await,
            "echo" => self.echo_command.execute(stream, &request.args).await,
            _ => Err(anyhow!("Unknown command: {}", request.command_name)),
        }
    }
}
