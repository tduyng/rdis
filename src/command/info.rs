use crate::{protocol::parser::RedisValue, replication::StreamType, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn execute(handler: &mut ResponseHandler) -> Result<()> {
        let mut response = String::new();

        match &handler.replica_info.stream_type {
            StreamType::Master => {
                response += "stream_type:master\r\n";
                response += &format!("master_replid:{}\r\n", handler.replica_info.master_replid);
                response += &format!(
                    "master_repl_offset:{}\r\n",
                    handler.replica_info.master_repl_offset
                );
            }
            StreamType::Slave => {
                response += "stream_type:slave\r\n";
            }
        }

        handler
            .write_response(RedisValue::bulk_string(response))
            .await?;
        Ok(())
    }
}
