use crate::stream::ResponseHandler;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PingCommand;

impl PingCommand {
    pub async fn execute(&self, handler: &mut ResponseHandler) -> Result<()> {
        handler.write_response("+PONG\r\n".to_string()).await?;
        Ok(())
    }
}
