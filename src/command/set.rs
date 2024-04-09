use super::RedisCommandInfo;
use crate::{protocol::parser::RespValue, stream::ResponseHandler, utils::current_time_ms};
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

        if command_contains_px_argument(command) {
            Self::execute_with_expiry(handler, command).await
        } else {
            Self::execute_set(handler, command).await
        }
    }

    async fn execute_set(handler: &mut ResponseHandler, command: &RedisCommandInfo) -> Result<()> {
        let key = command.args[0].clone();
        let value = command.args[1].clone();

        handler.database.set(key, value);
        handler
            .write_response(RespValue::SimpleString("OK".to_string()).encode())
            .await?;
        Ok(())
    }

    async fn execute_with_expiry(
        handler: &mut ResponseHandler,
        command: &RedisCommandInfo,
    ) -> Result<()> {
        let key = command.args[0].clone();
        let value = command.args[1].clone();

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
                    handler
                        .write_response(RespValue::SimpleString("OK".to_string()).encode())
                        .await?;
                    return Ok(());
                }
            }
        }

        Err(anyhow::anyhow!("Invalid expiry time"))
    }
}

fn command_contains_px_argument(command: &RedisCommandInfo) -> bool {
    command.args.iter().any(|arg| arg.to_lowercase() == "px")
}
