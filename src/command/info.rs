use crate::{
    protocol::parser::RespValue,
    stream::{RespHandler, StreamType},
};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn execute(handler: &RespHandler) -> Result<String> {
        let mut response = String::new();

        match &handler.stream_info.role {
            StreamType::Master => {
                response += "role:master\r\n";
                response += &format!("master_replid:{}\r\n", handler.stream_info.master_id);
                response += &format!(
                    "master_repl_offset:{}\r\n",
                    handler.stream_info.master_offset
                );
            }
            StreamType::Slave => {
                response += "role:slave\r\n";
            }
        }

        Ok(RespValue::BulkString(response).encode())
    }
}
