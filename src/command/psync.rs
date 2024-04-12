use crate::{protocol::parser::RespValue, stream::StreamInfo};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PsyncCommand;

impl PsyncCommand {
    pub async fn execute(stream_info: &StreamInfo) -> Result<String> {
        let full_resync =
            RespValue::SimpleString(format!("FULLRESYNC {} 0", stream_info.id)).encode();
        Ok(full_resync)
    }
}
