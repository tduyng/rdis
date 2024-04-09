use crate::{protocol::parser::RespValue, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PingCommand;

impl PingCommand {
    pub async fn execute(handler: &mut ResponseHandler) -> Result<()> {
        handler
            .write_response(RespValue::SimpleString("PONG".to_string()).encode())
            .await?;
        Ok(())
    }
}
