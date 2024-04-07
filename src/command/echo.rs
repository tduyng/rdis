use anyhow::{anyhow, Result};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug, Clone)]
pub struct EchoCommand;

impl EchoCommand {
    pub async fn execute(&self, stream: &mut TcpStream, args: Vec<String>) -> Result<()> {
        let response = match args.first() {
            Some(arg) => format!("${}\r\n{}\r\n", arg.len(), arg),
            None => return Err(anyhow!("Invalid arguments!")),
        };
        stream
            .write_all(response.as_bytes())
            .await
            .map_err(|e| anyhow!("Failed to write ECHO response to stream: {}", e))
    }
}
