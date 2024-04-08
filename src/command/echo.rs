use super::RedisCommand;
use crate::{protocol::parser::RedisValue, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct EchoCommand;

impl EchoCommand {
    pub async fn execute(
        &self,
        handler: &mut ResponseHandler,
        command: &RedisCommand,
    ) -> Result<()> {
        if let Some(arg) = command.args.first() {
            handler
                .write_value(RedisValue::BulkString(arg.clone()))
                .await?;
        } else {
            handler
                .write_value(RedisValue::BulkString("".to_string()))
                .await?;
        }
        Ok(())
    }
}
