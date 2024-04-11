use super::RedisCommandInfo;
use crate::store::RedisStore;
use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

#[derive(Debug, Clone)]
pub struct GetCommand;

impl GetCommand {
    pub async fn execute(
        mut stream: TcpStream,
        cmd_info: &RedisCommandInfo,
        store: &RwLock<RedisStore>,
    ) -> Result<()> {
        if cmd_info.args.len() != 1 {
            return Err(anyhow::anyhow!("GET command requires exactly one argument"));
        }
        let key = &cmd_info.args[0];
        let store = store.write().await;
        if let Some(value) = store.get(key) {
            let response = format!("${}\r\n{}\r\n", value.len(), value);
            stream.write_all(response.as_bytes()).await?;
        } else {
            stream.write_all("$-1\r\n".to_string().as_bytes()).await?;
        }
        Ok(())
    }
}
