use crate::{protocol::parser::RespValue, stream::RespHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PingCommand;

impl PingCommand {
    pub async fn execute(handler: &mut RespHandler) -> Result<()> {
        handler
            .write_response(RespValue::SimpleString("PONG".to_string()).encode())
            .await?;
        Ok(())
    }
}
