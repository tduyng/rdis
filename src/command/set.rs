use super::RedisCommandInfo;
use crate::{
    protocol::parser::RespValue,
    store::{Entry, RedisStore},
};
use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct SetCommand;

impl SetCommand {
    pub async fn execute(
        cmd_info: &mut RedisCommandInfo,
        store: &Arc<Mutex<RedisStore>>,
    ) -> Result<String> {
        let (key, value) = get_key_value(cmd_info)?;
        let expiry = get_expiry(cmd_info);
        let entry = Entry::new(value, expiry);

        store.lock().await.set(key, entry);

        Ok(RespValue::SimpleString("OK".to_string()).encode())
    }
}

pub fn get_key_value(cmd_info: &mut RedisCommandInfo) -> Result<(String, String)> {
    if cmd_info.args.len() < 2 {
        return Err(anyhow::anyhow!(
            "SET command requires exactly two arguments"
        ));
    }
    let key = cmd_info.args[0].clone();
    let value = cmd_info.args[1].clone();

    Ok((key, value))
}

pub fn get_expiry(cmd_info: &mut RedisCommandInfo) -> Option<Duration> {
    if cmd_info.args.len() < 4 {
        return None;
    }
    if let Some(tag) = cmd_info.args.get(2) {
        if tag != "px" {
            return None;
        }
    }
    if let Some(duration) = cmd_info.args.get(3) {
        let duration_time = duration.parse::<u64>().unwrap_or_default();
        return Some(Duration::from_millis(duration_time));
    }
    None
}
