use std::sync::Arc;

use super::RedisCommandInfo;
use crate::{protocol::parser::RespValue, store::RedisStore, utils::current_time_ms};
use anyhow::Result;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct SetCommand;

impl SetCommand {
    pub async fn execute(
        cmd_info: &mut RedisCommandInfo,
        store: &Arc<Mutex<RedisStore>>,
    ) -> Result<String> {
        if cmd_info.args.len() < 2 {
            return Err(anyhow::anyhow!(
                "SET command requires exactly two arguments"
            ));
        }

        let response = if contains_px_arg(cmd_info) {
            Self::execute_with_expiry(cmd_info, store).await
        } else {
            Self::execute_set(cmd_info, store).await
        };
        cmd_info.propagate(store).await?;

        response
    }

    async fn execute_set(
        cmd_info: &RedisCommandInfo,
        store: &Arc<Mutex<RedisStore>>,
    ) -> Result<String> {
        let key = cmd_info.args[0].clone();
        let value = cmd_info.args[1].clone();
        let mut store = store.lock().await;
        store.set(key, value);

        Ok(RespValue::SimpleString("OK".to_string()).encode())
    }

    async fn execute_with_expiry(
        cmd_info: &RedisCommandInfo,
        store: &Arc<Mutex<RedisStore>>,
    ) -> Result<String> {
        let key = cmd_info.args[0].clone();
        let value = cmd_info.args[1].clone();

        if let Some(px_index) = cmd_info
            .args
            .iter()
            .position(|arg| arg.to_lowercase() == "px")
        {
            if px_index + 1 < cmd_info.args.len() {
                if let Ok(expiry_ms) = cmd_info.args[px_index + 1].parse::<u128>() {
                    let current_time_ms = current_time_ms();
                    let expiry_time_ms = current_time_ms + expiry_ms;
                    let mut store = store.lock().await;
                    store.set_with_expiry(key, value, expiry_time_ms);
                    return Ok(RespValue::SimpleString("OK".to_string()).encode());
                }
            }
        }

        Err(anyhow::anyhow!("Invalid expiry time"))
    }
}

fn contains_px_arg(command: &RedisCommandInfo) -> bool {
    command.args.iter().any(|arg| arg.to_lowercase() == "px")
}
