use crate::protocol::parser::RespValue;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PingCommand;

impl PingCommand {
    pub async fn execute() -> Result<String> {
        Ok(RespValue::SimpleString("PONG".to_string()).encode())
    }
}
