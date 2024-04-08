use crate::stream::ResponseHandler;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ReplConfCommand;

impl ReplConfCommand {
    pub async fn execute(&self, handler: &mut ResponseHandler) -> Result<()> {
        handler.write_response("+OK\r\n".to_string()).await?;
        Ok(())
    }
}
