use super::RedisCommand;
use crate::stream::ResponseHandler;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct GetCommand;

impl GetCommand {
    pub async fn execute(
        &self,
        handler: &mut ResponseHandler,
        command: &RedisCommand,
    ) -> Result<()> {
        if command.args.len() != 1 {
            return Err(anyhow::anyhow!("GET command requires exactly one argument"));
        }
        let key = &command.args[0];

        if let Some(value) = handler.database.get(key) {
            let response = format!("${}\r\n{}\r\n", value.len(), value);
            handler.write_response(response).await?;
        } else {
            handler.write_response("$-1\r\n".to_string()).await?;
        }
        Ok(())
    }
}
