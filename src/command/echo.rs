use super::RedisCommandInfo;
use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug, Clone)]
pub struct EchoCommand;

impl EchoCommand {
    pub async fn execute(mut stream: TcpStream, cmd_info: &RedisCommandInfo) -> Result<()> {
        if cmd_info.args.is_empty() {
            return Err(anyhow::anyhow!(
                "ECHO command requires at least one argument"
            ));
        }
        let message = cmd_info.args.join(" ");
        stream
            .write_all(format!("+{}\r\n", message).as_bytes())
            .await?;
        Ok(())
    }
}
