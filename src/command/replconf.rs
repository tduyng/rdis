use crate::protocol::parser::RespValue;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ReplConfCommand;

impl ReplConfCommand {
    pub async fn execute() -> Result<String> {
        Ok(RespValue::SimpleString("OK".to_string()).encode())
    }
}
