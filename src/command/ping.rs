use anyhow::{anyhow, Result};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug, Clone)]
pub struct Ping;

impl Ping {
    pub async fn execute(&self, stream: &mut TcpStream) -> Result<()> {
        stream
            .write_all(b"+PONG\r\n")
            .await
            .map_err(|e| anyhow!("Failed to write PONG response to stream: {}", e))
    }
}
