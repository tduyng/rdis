use crate::protocol::parser::RespValue;
use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug, Clone)]
pub struct ReplConfCommand;

impl ReplConfCommand {
    pub async fn execute(mut stream: TcpStream) -> Result<()> {
        stream
            .write_all(
                RespValue::SimpleString("OK".to_string())
                    .encode()
                    .as_bytes(),
            )
            .await?;
        Ok(())
    }
}
