use crate::{protocol::parser::RespValue, stream::RespHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PsyncCommand;

impl PsyncCommand {
    pub async fn execute(handler: &mut RespHandler) -> Result<String> {
        let full_resync =
            RespValue::SimpleString(format!("FULLRESYNC {} 0", handler.stream_info.master_id))
                .encode();
        Ok(full_resync)
    }
}
