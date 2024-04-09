use crate::{protocol::parser::RespValue, replication::StreamType, stream::RespHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn execute(handler: &mut RespHandler) -> Result<()> {
        let mut response = String::new();

        match &handler.replica_info.role {
            StreamType::Master => {
                response += "role:master\r\n";
                response += &format!("master_replid:{}\r\n", handler.replica_info.master_replid);
                response += &format!(
                    "master_repl_offset:{}\r\n",
                    handler.replica_info.master_repl_offset
                );
            }
            StreamType::Slave => {
                response += "role:slave\r\n";
            }
        }

        handler
            .write_response(RespValue::BulkString(response).encode())
            .await?;
        Ok(())
    }
}
