use crate::{protocol::parser::RedisValue, stream::ResponseHandler};
use anyhow::Result;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct PsyncCommand;

impl PsyncCommand {
    pub async fn execute(handler: &mut ResponseHandler) -> Result<()> {
        let full_resync = RedisValue::simple_string(format!(
            "FULLRESYNC {} 0",
            handler.replica_info.master_replid
        ));

        let empty_rdb_hex = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
        let empty_rdb_bytes = hex::decode(empty_rdb_hex)?;
        let message = format!("${}\r\n", empty_rdb_bytes.len());

        handler.write_response(full_resync).await?;
        handler.write_response(message).await?;
        handler.stream.write_all(&empty_rdb_bytes).await?;
        Ok(())
    }
}
