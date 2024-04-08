use crate::{protocol::parser::RedisValue, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn execute(&self, handler: &mut ResponseHandler) -> Result<()> {
        handler
            .write_response(RedisValue::bulk_string("role:master".to_string()))
            .await?;
        Ok(())
    }
}
