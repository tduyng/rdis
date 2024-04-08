use crate::{protocol::parser::RedisValue, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ReplConfCommand;

impl ReplConfCommand {
    pub async fn execute(handler: &mut ResponseHandler) -> Result<()> {
        handler
            .write_response(RedisValue::simple_string("OK".to_string()))
            .await?;
        Ok(())
    }
}
