use std::sync::Arc;

use super::RedisCommandInfo;
use crate::store::RedisStore;
use anyhow::Result;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct GetCommand;

impl GetCommand {
    pub async fn execute(
        cmd_info: &RedisCommandInfo,
        store: &Arc<Mutex<RedisStore>>,
    ) -> Result<String> {
        if cmd_info.args.len() != 1 {
            return Err(anyhow::anyhow!("GET command requires exactly one argument"));
        }
        let key = &cmd_info.args[0];
        let store = store.lock().await;
        if let Some(value) = store.get(key) {
            let response = format!("${}\r\n{}\r\n", value.len(), value);
            Ok(response)
        } else {
            Ok("$-1\r\n".to_string())
        }
    }
}
