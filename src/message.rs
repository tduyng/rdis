use anyhow::{anyhow, Result};
use bytes::BytesMut;

use crate::{
    command::CommandInfo,
    protocol::parser::{parse_int, read_until_crlf},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Simple(String),
    Bulk(String),
    Array(Vec<Message>),
    Int(isize),
    Error(String),
}

impl Message {
    pub fn decode(buffer: BytesMut) -> Result<(Message, usize)> {
        match buffer[0] as char {
            '+' => parse_simple_string(buffer),
            '*' => parse_array(buffer),
            '$' => parse_bulk_string(buffer),
            _ => Err(anyhow!("Not a known value type {:?}", buffer)),
        }
    }

    pub fn encode(&self) -> String {
        match self {
            Message::Simple(value) => {
                format!("+{}\r\n", value)
            }
            Message::Bulk(value) => {
                format!("${}\r\n{}\r\n", value.chars().count(), value)
            }
            Message::Array(values) => {
                let mut result = format!("*{}\r\n", values.len());
                for value in values {
                    result.push_str(&value.encode());
                }
                result
            }
            Message::Int(value) => {
                format!(":{}\r\n", value)
            }
            Message::Error(s) => format!("-{}\r\n", s),
        }
    }

    pub fn encode_array_str(values: Vec<&str>) -> String {
        let mut result = format!("*{}\r\n", values.len());
        for value in values {
            result.push_str(&Message::Bulk(value.to_string()).encode());
        }
        result
    }

    pub async fn parse_command(message: Message) -> Result<CommandInfo> {
        match message {
            Message::Array(a) => {
                if let Some(name) = a.first().and_then(|v| unpack_bulk_str(v.clone())) {
                    let args: Vec<String> = a.into_iter().skip(1).filter_map(unpack_bulk_str).collect();
                    Ok(CommandInfo::new(name, args))
                } else {
                    Err(anyhow!("Invalid command format"))
                }
            }
            _ => Err(anyhow!("Unexpected command format: {:?}", message)),
        }
    }
}

pub fn parse_simple_string(buffer: BytesMut) -> Result<(Message, usize)> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();
        return Ok((Message::Simple(string), len + 1));
    }
    Err(anyhow!("Invalid string {:?}", buffer))
}

pub fn parse_array(buffer: BytesMut) -> Result<(Message, usize)> {
    let (array_length, mut bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let array_length = parse_int(line)?;
        (array_length, len + 1)
    } else {
        return Err(anyhow!("Invalid array format {:?}", buffer));
    };
    let mut items = vec![];
    for _ in 0..array_length {
        let (array_item, len) = Message::decode(BytesMut::from(&buffer[bytes_consumed..]))?;
        items.push(array_item);
        bytes_consumed += len;
    }
    Ok((Message::Array(items), bytes_consumed))
}

pub fn parse_bulk_string(buffer: BytesMut) -> Result<(Message, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let bulk_str_len = parse_int(line)?;
        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow!("Invalid array format {:?}", buffer));
    };
    let end_of_bulk_str = bytes_consumed + bulk_str_len;
    let total_parsed = end_of_bulk_str + 2;
    Ok((
        Message::Bulk(String::from_utf8(buffer[bytes_consumed..end_of_bulk_str].to_vec())?),
        total_parsed,
    ))
}

pub fn unpack_bulk_str(value: Message) -> Option<String> {
    match value {
        Message::Bulk(s) => Some(s),
        _ => None,
    }
}
