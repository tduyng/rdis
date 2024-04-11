use crate::{protocol::parser::RespValue, replica::StreamType, stream::RespHandler};
use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug, Clone)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn execute(mut stream: TcpStream, handler: &RespHandler) -> Result<()> {
        let mut response = String::new();

        match &handler.repl_info.role {
            StreamType::Master => {
                response += "role:master\r\n";
                response += &format!("master_replid:{}\r\n", handler.repl_info.master_id);
                response += &format!("master_repl_offset:{}\r\n", handler.repl_info.master_offset);
            }
            StreamType::Slave => {
                response += "role:slave\r\n";
            }
        }

        stream
            .write_all(RespValue::BulkString(response).encode().as_bytes())
            .await?;
        Ok(())
    }
}
