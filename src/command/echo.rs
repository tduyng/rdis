use super::RedisCommandInfo;
use crate::stream::ResponseHandler;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct EchoCommand;

impl EchoCommand {
    pub async fn execute(handler: &mut ResponseHandler, command: &RedisCommandInfo) -> Result<()> {
        if command.args.is_empty() {
            return Err(anyhow::anyhow!(
                "ECHO command requires at least one argument"
            ));
        }
        let message = command.args.join(" ");
        handler.write_response(format!("+{}\r\n", message)).await?;
        Ok(())
    }
}
