use crate::command::RedisCommand;
use anyhow::{anyhow, Result};
use bytes::{Buf, BytesMut};

#[derive(Clone, Debug)]
pub enum RedisValue {
    SimpleString(String),
    BulkString(String),
    Array(Vec<RedisValue>),
}

impl RedisValue {
    pub fn serialize(&self) -> String {
        match self {
            RedisValue::SimpleString(s) => format!("+{}\r\n", s),
            RedisValue::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
            RedisValue::Array(vals) => {
                let serialized_vals: Vec<String> = vals.iter().map(|v| v.serialize()).collect();
                format!("*{}\r\n{}", vals.len(), serialized_vals.join("\r\n"))
            }
        }
    }
}

pub fn parse_command(buffer: &mut BytesMut) -> Result<RedisCommand> {

    match parse_message(buffer)? {
        RedisValue::Array(vals) => {
            let mut args = vec![];
            for val in vals {
                if let RedisValue::BulkString(s) = val {
                    args.push(s);
                } else {
                    return Err(anyhow!("Invalid command syntax, expected bulk string"));
                }
            }
            if args.is_empty() {
                return Err(anyhow!("Invalid command syntax, expected non-empty array"));
            }
            Ok(RedisCommand {
                name: args[0].clone(),
                args: args[1..].to_vec(),
            })
        }
        _ => Err(anyhow!("Invalid command syntax, expected array")),
    }
}

pub fn parse_message(buffer: &mut BytesMut) -> Result<RedisValue> {
    match buffer[0] as char {
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        '+' => parse_simple_string(buffer),
        _ => Err(anyhow!("Improper RESP format")),
    }
}

fn parse_simple_string(buffer: &mut BytesMut) -> Result<RedisValue> {
    assert_eq!(buffer[0] as char, '+');
    buffer.advance(1);
    if let Some((line, len)) = read_until_crlf(buffer) {
        let string = String::from_utf8(line.to_vec())?;
        buffer.advance(len);
        Ok(RedisValue::SimpleString(string))
    } else {
        Err(anyhow!("Invalid string"))
    }
}

fn parse_array(buffer: &mut BytesMut) -> Result<RedisValue> {
    assert_eq!(buffer[0] as char, '*');
    buffer.advance(1);
    if let Some((line, len)) = read_until_crlf(buffer) {
        let array_length: usize = String::from_utf8(line.to_vec())?.parse()?;
        buffer.advance(len);
        let mut items = Vec::with_capacity(array_length);
        for _ in 0..array_length {
            let item = parse_message(buffer)?;
            items.push(item);
        }
        Ok(RedisValue::Array(items))
    } else {
        Err(anyhow!("Invalid array format"))
    }
}

fn parse_bulk_string(buffer: &mut BytesMut) -> Result<RedisValue> {
    assert_eq!(buffer[0] as char, '$');
    buffer.advance(1);
    if let Some((line, len)) = read_until_crlf(buffer) {
        let bulk_str_len: usize = String::from_utf8(line.to_vec())?.parse()?;
        buffer.advance(len);
        if buffer.len() < bulk_str_len {
            return Err(anyhow!("Invalid bulk string"));
        }
        let value = String::from_utf8(buffer[..bulk_str_len].to_vec())?;
        buffer.advance(bulk_str_len + 2); // +2 to skip CRLF
        Ok(RedisValue::BulkString(value))
    } else {
        Err(anyhow!("Invalid bulk string"))
    }
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }
    None
}
