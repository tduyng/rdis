use crate::command::CommandRegistry;
use crate::protocol::parser::{read_resp_value, RespValue};
use anyhow::Result;
use tokio::net::TcpStream;

pub async fn handle_stream(mut stream: TcpStream, redis_command: CommandRegistry) -> Result<()> {
    println!("Accepted new connection");

    loop {
        let resp_value = read_resp_value(&mut stream).await?;
        match resp_value {
            RespValue::Array(mut values) => {
                if let Some(RespValue::BulkString(Some(command_str))) = values.pop() {
                    let command_string = String::from_utf8_lossy(&command_str).to_string();
                    let parts: Vec<&str> = command_string.split_whitespace().collect();
                    if !parts.is_empty() {
                        let command_name = parts[0].to_lowercase();
                        let args = parts[1..].iter().map(|&s| s.to_string()).collect();
                        redis_command
                            .execute(&command_name, &mut stream, args)
                            .await?;
                    }
                }
            }
            _ => {
                println!("Invalid RESP value received");
            }
        }
    }
}
