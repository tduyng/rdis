use super::RedisCommand;
use crate::stream::ResponseHandler;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct EchoCommand;

impl EchoCommand {
    pub async fn execute(
        &self,
        handler: &mut ResponseHandler,
        command: &RedisCommand,
    ) -> Result<()> {
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
