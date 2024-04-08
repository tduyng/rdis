use crate::{protocol::parser::RedisValue, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PingCommand;

impl PingCommand {
    pub async fn execute(handler: &mut ResponseHandler) -> Result<()> {
        handler
            .write_response(RedisValue::simple_string("PONG".to_string()))
            .await?;
        Ok(())
    }
}
