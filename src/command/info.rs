use crate::{protocol::parser::RedisValue, replication::ReplicaMode, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn execute(&self, handler: &mut ResponseHandler) -> Result<()> {
        let mut response = String::new();

        match &handler.replica_mode {
            ReplicaMode::Master => {
                response += "role:master\r\n";
                response += "master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb\r\n";
                response += "master_repl_offset:0\r\n";
            }
            ReplicaMode::Slave(_) => {
                response += "role:slave\r\n";
            }
        }

        handler
            .write_response(RedisValue::bulk_string(response))
            .await?;
        Ok(())
    }
}
