use super::RedisCommand;
use crate::stream::ResponseHandler;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SetCommand;

impl SetCommand {
    pub async fn execute(
        &self,
        handler: &mut ResponseHandler,
        command: &RedisCommand,
    ) -> Result<()> {
        if command.args.len() != 2 {
            return Err(anyhow::anyhow!(
                "SET command requires exactly two arguments"
            ));
        }
        let key = command.args[0].clone();
        let value = command.args[1].clone();
        handler.database.set(key, value);
        handler.write_response("+OK\r\n".to_string()).await?;
        Ok(())
    }
}
