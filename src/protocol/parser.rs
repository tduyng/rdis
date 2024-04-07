use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use tokio::{io::AsyncReadExt, net::TcpStream};

pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Vec<RespValue>),
}

pub async fn read_resp_value(stream: &mut TcpStream) -> Result<RespValue> {
    let mut buffer = Vec::new();
    let mut is_reading_bulk_string = false;
    let mut bulk_string_length = 0;

    loop {
        let mut read_buf = [0; 1024];
        let bytes_read = stream.read(&mut read_buf).await?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&read_buf[..bytes_read]);

        let mut command_buffer = VecDeque::new();
        while let Some(idx) = buffer.iter().position(|&b| b == b'\n') {
            let command = buffer.drain(..idx + 1).collect::<Vec<_>>();
            command_buffer.push_back(command);
        }

        while let Some(mut command) = command_buffer.pop_front() {
            if let Some(&b'\r') = command.last() {
                command.pop();
            }
            if let Some(&b'\n') = command.last() {
                command.pop();
            }

            // Parsing RESP value
            let resp_value = parse_resp_value(
                &command,
                &mut is_reading_bulk_string,
                &mut bulk_string_length,
            )?;

            // Check if the parsing is complete
            match resp_value {
                Some(value) => return Ok(value),
                None => continue,
            }
        }
    }

    Err(anyhow!("Incomplete RESP value"))
}

fn parse_resp_value(
    command: &[u8],
    is_reading_bulk_string: &mut bool,
    bulk_string_length: &mut usize,
) -> Result<Option<RespValue>> {
    if *is_reading_bulk_string {
        *bulk_string_length -= command.len();
        if *bulk_string_length > 0 {
            // Continue reading bulk string
            return Ok(None);
        } else {
            // Bulk string reading is complete
            *is_reading_bulk_string = false;
            return Ok(Some(RespValue::BulkString(Some(command.to_vec()))));
        }
    }

    match command.first() {
        Some(&b'+') => {
            let value = String::from_utf8_lossy(&command[1..]).to_string();
            Ok(Some(RespValue::SimpleString(value)))
        }
        Some(&b'-') => {
            let error_message = String::from_utf8_lossy(&command[1..]).to_string();
            Ok(Some(RespValue::Error(error_message)))
        }
        Some(&b':') => {
            let value = String::from_utf8_lossy(&command[1..]).parse::<i64>()?;
            Ok(Some(RespValue::Integer(value)))
        }
        Some(&b'$') => {
            let length = String::from_utf8_lossy(&command[1..]).parse::<usize>()?;
            if length == -1isize as usize {
                Ok(Some(RespValue::BulkString(None)))
            } else {
                *is_reading_bulk_string = true;
                *bulk_string_length = length;
                Ok(None)
            }
        }
        Some(&b'*') => {
            let length = String::from_utf8_lossy(&command[1..]).parse::<usize>()?;
            let mut array = Vec::with_capacity(length);
            for _ in 0..length {
                array.push(RespValue::BulkString(None));
            }
            Ok(Some(RespValue::Array(array)))
        }
        _ => Err(anyhow!("Invalid RESP value")),
    }
}
