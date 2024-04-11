use self::{
    echo::EchoCommand, get::GetCommand, info::InfoCommand, ping::PingCommand,
    replconf::ReplConfCommand, set::SetCommand,
};
use crate::{protocol::parser::RespValue, store::RedisStore, stream::RespHandler};
use anyhow::{anyhow, Result};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

mod echo;
mod get;
mod info;
mod ping;
mod replconf;
mod set;

pub struct RedisCommandInfo {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RedisCommand {}

impl RedisCommand {
    pub async fn execute(
        stream: TcpStream,
        handler: &mut RespHandler,
        cmd_info: &RedisCommandInfo,
        store: &RwLock<RedisStore>,
    ) -> Result<()> {
        match cmd_info.name.to_lowercase().as_str() {
            "ping" => PingCommand::execute(stream).await,
            "echo" => EchoCommand::execute(stream, cmd_info).await,
            "get" => GetCommand::execute(stream, cmd_info, store).await,
            "set" => SetCommand::execute(stream, cmd_info, store).await,
            "info" => InfoCommand::execute(stream, &handler).await,
            "replconf" => ReplConfCommand::execute(stream).await,
            _ => Err(anyhow!("Unknown command: {}", cmd_info.name)),
        }
    }
}

impl RedisCommandInfo {
    pub fn new(name: String, args: Vec<String>) -> Self {
        RedisCommandInfo { name, args }
    }

    pub fn encode(&self) -> Vec<RespValue> {
        let mut array_values = Vec::with_capacity(self.args.len() + 1);
        array_values.push(RespValue::BulkString(self.name.clone()));
        for arg in &self.args {
            array_values.push(RespValue::BulkString(arg.clone()));
        }
        array_values
    }

    pub async fn propagate(&mut self, store: &RwLock<RedisStore>) -> Result<()> {
        let encoded_command = RespValue::Array(self.encode()).encode();
        let mut store = store.write().await;
        for stream in store.repl_streams.iter_mut() {
            stream.write_all(encoded_command.as_bytes()).await?;
        }
        Ok(())
    }
}
