use crate::{protocol::parser::RedisValue, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PingCommand;

impl PingCommand {
    pub async fn execute(&self, handler: &mut ResponseHandler) -> Result<()> {
        handler
            .write_value(RedisValue::SimpleString("PONG".to_string()))
            .await?;
        Ok(())
    }
}
