use super::RedisCommandInfo;
use crate::{stream::ResponseHandler, utils::current_time_ms};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SetCommand;

impl SetCommand {
    pub async fn execute(handler: &mut ResponseHandler, command: &RedisCommandInfo) -> Result<()> {
        if command.args.len() < 2 {
            return Err(anyhow::anyhow!(
                "SET command requires exactly two arguments"
            ));
        }
        let key = command.args[0].clone();
        let value = command.args[1].clone();

        // Check if the command includes the "PX" argument
        if let Some(px_index) = command
            .args
            .iter()
            .position(|arg| arg.to_lowercase() == "px")
        {
            if px_index + 1 < command.args.len() {
                if let Ok(expiry_ms) = command.args[px_index + 1].parse::<u128>() {
                    let current_time_ms = current_time_ms();
                    let expiry_time_ms = current_time_ms + expiry_ms;
                    handler.database.set_with_expiry(key, value, expiry_time_ms);
                    handler.write_response("+OK\r\n".to_string()).await?;
                    return Ok(());
                }
            }
            return Err(anyhow::anyhow!("Invalid expiry time"));
        }

        handler.database.set(key, value);
        handler.write_response("+OK\r\n".to_string()).await?;
        Ok(())
    }
}
