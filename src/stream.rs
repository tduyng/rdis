use crate::command::CommandRegistry;
use crate::protocol::parser::{read_resp_value, RespValue};
use anyhow::{anyhow, Result};
use tokio::net::TcpStream;

pub struct RedisRequest {
    pub command_name: String,
    pub args: Option<Vec<String>>,
}

pub async fn handle_stream(mut stream: TcpStream, redis_command: CommandRegistry) -> Result<()> {
    println!("Accepted new connection");

    loop {
        let resp_value = read_resp_value(&mut stream).await?;
        match resp_value {
            RespValue::Array(mut values) => {
                if let Some(RespValue::BulkString(Some(command_bytes))) = values.pop() {
                    if let Ok(command_str) = String::from_utf8(command_bytes) {
                        let request = parse_request(&command_str)?;
                        redis_command.execute(&mut stream, &request).await?;
                    }
                }
            }
            _ => {
                println!("Invalid RESP value received");
            }
        }
    }
}

fn parse_request(raw_request: &str) -> Result<RedisRequest> {
    let parts: Vec<&str> = raw_request.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty request"));
    }
    let command_name = parts[0].to_lowercase();
    let args = if parts.len() > 1 {
        Some(parts[1..].iter().map(|&s| s.to_string()).collect())
    } else {
        None
    };

    Ok(RedisRequest {
        command_name,
        args,
    })
}
