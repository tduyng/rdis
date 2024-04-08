use crate::{protocol::parser::RedisValue, replication::ReplicaMode, stream::ResponseHandler};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn execute(&self, handler: &mut ResponseHandler) -> Result<()> {
        let response = match &handler.replica_mode {
            ReplicaMode::Master => RedisValue::bulk_string("role:master".to_string()),
            ReplicaMode::Slave(_) => RedisValue::bulk_string("role:slave".to_string()),
        };

        handler.write_response(response).await?;
        Ok(())
    }
}
