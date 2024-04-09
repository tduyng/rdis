use crate::{
    protocol::{parser::RespValue, rdb::Rdb},
    stream::RespHandler,
};
use anyhow::Result;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct PsyncCommand;

impl PsyncCommand {
    pub async fn execute(handler: &mut RespHandler) -> Result<()> {
        let full_resync =
            RespValue::SimpleString(format!("FULLRESYNC {} 0", handler.replica_info.repl_id))
                .encode();
        handler.write_response(full_resync).await?;

        let empty_rdb = Rdb::get_empty();
        handler.stream.write_all(&empty_rdb).await?;
        Ok(())
    }
}
