use crate::database::Database;
use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use super::RedisCommand;
#[derive(Debug, Clone)]
pub struct GetCommand;

impl GetCommand {
    pub async fn execute(
        &self,
        stream: &mut TcpStream,
        database: &Database,
        command: &RedisCommand,
    ) -> Result<()> {
        if command.args.len() != 2 {
            return Err(anyhow::anyhow!("GET command requires exactly one argument"));
        }
        let key = &command.args[1];

        if let Some(value) = database.get(key) {
            let response = format!("${}\r\n{}\r\n", value.len(), value);
            stream.write_all(response.as_bytes()).await?;
        } else {
            stream.write_all(b"$-1\r\n").await?;
        }
        Ok(())
    }
}
