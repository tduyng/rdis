use crate::{protocol::parser::RedisValue, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PsyncCommand;

impl PsyncCommand {
    pub async fn execute(handler: &mut ResponseHandler) -> Result<()> {
        handler
            .write_response(RedisValue::simple_string(format!(
                "FULLRESYNC {} {}",
                handler.replica_info.master_replid, handler.replica_info.master_repl_offset
            )))
            .await?;
        Ok(())
    }
}
