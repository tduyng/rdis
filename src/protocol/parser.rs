use crate::command::RedisCommand;
use anyhow::{anyhow, Result};

fn parse_simple_string(buffer: &[u8]) -> Result<(usize, String)> {
    assert!(buffer[0] == b'+');
    let term = buffer.iter().position(|&x| x == b'\r').unwrap();
    let value = String::from_utf8(buffer[1..term].to_vec()).unwrap();
    // +2 to get past the trailing \r\n
    Ok((term + 2, value))
}

fn parse_bulk_string(buffer: &[u8]) -> Result<(usize, String)> {
    assert!(buffer[0] == b'$');
    let sep = buffer.iter().position(|&x| x == b'\r').unwrap();
    let string_len = String::from_utf8(buffer[1..sep].to_vec())
        .unwrap()
        .parse::<usize>()
        .unwrap();
    if string_len == 0 {
        // Empty string: $0\r\n\r\n -> sep points to first \r, so + 3 to get to the end
        return Ok((sep + 4, String::new()));
    }
    let string_start = sep + 2; // sep is \r, so +2 to get to the start of the actual string
    let string_end = string_start + string_len;
    let value = String::from_utf8(buffer[string_start..string_end].to_vec()).unwrap();
    // +2 to get past the trailing \r\n
    Ok((string_end + 2, value))
}

fn parse_array(buffer: &[u8]) -> Result<(usize, Vec<String>)> {
    assert!(buffer[0] == b'*');
    let sep = buffer.iter().position(|&x| x == b'\r').unwrap();
    let array_length = String::from_utf8(buffer[1..sep].to_vec())
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let mut pos = sep + 2;
    let mut params = Vec::<String>::with_capacity(array_length);

    for _ in 0..array_length {
        match buffer[pos] {
            b'$' => {
                let (bytes, value) = parse_bulk_string(&buffer[pos..])?;
                params.push(value);
                pos += bytes;
            }
            b'+' => {
                let (bytes, value) = parse_simple_string(&buffer[pos..])?;
                params.push(value);
                pos += bytes;
            }
            _ => {
                return Err(anyhow!(
                    "Invalid command syntax, expected bulk string or simple string, got {}",
                    buffer[pos]
                ))
            }
        };
    }

    Ok((pos, params))
}

pub fn parse_command(buffer: &[u8]) -> Result<RedisCommand> {
    if buffer.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    if buffer[0] != b'*' {
        return Err(anyhow!(
            "Invalid command syntax, expected array, got {}",
            buffer[0]
        ));
    };

    let (_pos, params) = parse_array(buffer)?;

    if params.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    Ok(RedisCommand {
        name: params[0].clone(),
        args: params[1..].to_vec(),
    })
}
