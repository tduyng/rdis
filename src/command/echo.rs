use super::RedisCommand;
use anyhow::{anyhow, Result};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug, Clone)]
pub struct EchoCommand;

impl EchoCommand {
    pub async fn execute(&self, stream: &mut TcpStream, command: &RedisCommand) -> Result<()> {
        if command.args.len() != 1 {
            return Err(anyhow!("ECHO command requires exactly one argument"));
        }

        stream
            .write_all(format!("+{}\r\n", command.args[0]).as_bytes())
            .await
            .map_err(|e| anyhow!("Failed to write ECHO response to stream: {}", e))
    }
}
