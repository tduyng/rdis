use crate::{protocol::parser::RedisValue, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PsyncCommand;

impl PsyncCommand {
    pub async fn execute(handler: &mut ResponseHandler) -> Result<()> {
        handler
            .write_response(RedisValue::simple_string(
                "FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0".to_string(),
            ))
            .await?;
        Ok(())
    }
}
