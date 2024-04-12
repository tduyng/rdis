use crate::{protocol::parser::RespValue, stream::StreamInfo};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn execute(info: &StreamInfo) -> Result<String> {
        let response = format!(
            "# Replication\n\
            role:{}\n\
            connected_clients:{}\n\
            master_replid:{}\n\
            master_repl_offset:{}\n\
            ",
            info.role, info.connected_clients, info.id, info.offset
        );

        Ok(RespValue::BulkString(response).encode())
    }
}
