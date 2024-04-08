use anyhow::{anyhow, Result};
use bytes::BytesMut;

#[derive(Clone, Debug)]
pub enum RedisValue {
    SimpleString(String),
    BulkString(String),
    Array(Vec<RedisValue>),
}

pub fn parse_message(buffer: BytesMut) -> Result<(RedisValue, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(anyhow!("Not a known value type {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(RedisValue, usize)> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();
        return Ok((RedisValue::SimpleString(string), len + 1));
    }
    Err(anyhow!("Invalid string {:?}", buffer))
}

fn parse_array(buffer: BytesMut) -> Result<(RedisValue, usize)> {
    let (array_length, mut bytes_consumed) =
        if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
            let array_length = parse_int(line)?;
            (array_length, len + 1)
        } else {
            return Err(anyhow!("Invalid array format {:?}", buffer));
        };
    let mut items = vec![];
    for _ in 0..array_length {
        let (array_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;
        items.push(array_item);
        bytes_consumed += len;
    }
    Ok((RedisValue::Array(items), bytes_consumed))
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(RedisValue, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let bulk_str_len = parse_int(line)?;
        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow!("Invalid array format {:?}", buffer));
    };
    let end_of_bulk_str = bytes_consumed + bulk_str_len;
    let total_parsed = end_of_bulk_str + 2;
    Ok((
        RedisValue::BulkString(String::from_utf8(
            buffer[bytes_consumed..end_of_bulk_str].to_vec(),
        )?),
        total_parsed,
    ))
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }
    None
}

fn parse_int(buffer: &[u8]) -> Result<usize> {
    Ok(String::from_utf8(buffer.to_vec())?.parse()?)
}
