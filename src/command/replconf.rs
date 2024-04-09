use crate::{protocol::parser::RespValue, stream::RespHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ReplConfCommand;

impl ReplConfCommand {
    pub async fn execute(handler: &mut RespHandler) -> Result<()> {
        handler
            .write_response(RespValue::SimpleString("OK".to_string()).encode())
            .await?;
        Ok(())
    }
}
