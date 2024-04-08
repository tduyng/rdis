use crate::database::Database;
use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use super::RedisCommand;

#[derive(Debug, Clone)]
pub struct SetCommand;

impl SetCommand {
    pub async fn execute(
        &self,
        stream: &mut TcpStream,
        database: &mut Database,
        command: &RedisCommand,
    ) -> Result<()> {
        if command.args.len() != 3 {
            return Err(anyhow::anyhow!(
                "SET command requires exactly two arguments"
            ));
        }
        let key = command.args[1].clone();
        let value = command.args[2].clone();
        database.set(key, value);
        stream.write_all(b"+Ok\r\n").await?;
        Ok(())
    }
}
