use super::RedisCommandInfo;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct EchoCommand;

impl EchoCommand {
    pub async fn execute(cmd_info: &RedisCommandInfo) -> Result<String> {
        if cmd_info.args.is_empty() {
            return Err(anyhow::anyhow!(
                "ECHO command requires at least one argument"
            ));
        }
        let message = cmd_info.args.join(" ");
        Ok(format!("+{}\r\n", message))
    }
}
