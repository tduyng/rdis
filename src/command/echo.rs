use anyhow::{anyhow, Result};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug, Clone)]
pub struct EchoCommand;

impl EchoCommand {
    pub async fn execute(&self, stream: &mut TcpStream, args: &Option<Vec<String>>) -> Result<()> {
        let response = match args {
            Some(ref args) if !args.is_empty() => {
                let arg = &args[0];
                format!("${}\r\n{}\r\n", arg.len(), arg)
            }
            Some(_) => return Err(anyhow!("Empty argument list for ECHO command")),
            None => return Err(anyhow!("No arguments provided for ECHO command")),
        };

        stream
            .write_all(response.as_bytes())
            .await
            .map_err(|e| anyhow!("Failed to write ECHO response to stream: {}", e))
    }
}
