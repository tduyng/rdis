use anyhow::Result;

pub fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }
    None
}

pub fn parse_int(buffer: &[u8]) -> Result<usize> {
    Ok(String::from_utf8(buffer.to_vec())?.parse()?)
}
