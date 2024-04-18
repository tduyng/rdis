use crate::{
    store::{Entry, Store},
    stream::StreamInfo,
};
use std::{env, path::Path, sync::Arc};
use tokio::{fs::File, io::AsyncReadExt};

pub struct Rdb {}

impl Rdb {
    pub fn get_empty() -> Vec<u8> {
        let empty_writer = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
        hex_to_bytes(empty_writer)
    }

    pub async fn read_file(stream_info: &Arc<StreamInfo>) -> Option<Vec<u8>> {
        let config = stream_info.config.lock().await;
        let directory = config
            .dir
            .clone()
            .unwrap_or_else(|| env::current_dir().unwrap().into_os_string().into_string().unwrap());
        let filename = config.dbfilename.as_ref()?;
        let path = Path::new(&directory).join(filename);
        let mut file = match File::open(&path).await {
            Ok(file) => file,
            Err(_) => return None,
        };
        let mut buffer = Vec::new();
        if (file.read_to_end(&mut buffer).await).is_err() {
            return None;
        }
        Some(buffer)
    }

    pub fn parse_rdb(store: &mut Store, data: &[u8]) {
        let mut marker = 0;
        if !has_magic_number(data, &mut marker) {
            return;
        }
        if !find_database_selector(data, 0x00, &mut marker) {
            return;
        }
        if !read_resizedb_field(data, &mut marker) {
            return;
        }
        read_value(store, data, &mut marker);
        println!("{:2x}", data[marker]);
    }
}

#[derive(Debug)]
pub struct RdbConfig {
    pub dir: Option<String>,
    pub dbfilename: Option<String>,
}

impl Default for RdbConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RdbConfig {
    pub fn new() -> Self {
        RdbConfig {
            dir: None,
            dbfilename: None,
        }
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        match key.to_lowercase().as_str() {
            "dir" => self.dir.clone(),
            "dbfilename" => self.dbfilename.clone(),
            _ => None,
        }
    }
}

fn hex_to_bytes(content: &str) -> Vec<u8> {
    let byte_content = hex::decode(content).unwrap();
    let mut result = format!("${}\r\n", byte_content.len()).into_bytes();
    result.extend(byte_content);
    result
}

fn has_magic_number(data: &[u8], marker: &mut usize) -> bool {
    let magic_number = b"REDIS";
    *marker += magic_number.len();
    magic_number == &data[0..magic_number.len()]
}

fn find_database_selector(data: &[u8], database: u8, marker: &mut usize) -> bool {
    let start = *marker + 1;
    for i in start..data.len() {
        if data[i - 1] == 0xFE && data[i] == database {
            *marker = i + 1;
            return true;
        }
    }
    false
}

fn read_length_encoded_int(data: &[u8], marker: &mut usize) -> Option<u64> {
    let original = data[*marker];
    let tag = original & 0x03;
    *marker += 1;
    match tag {
        0b11 => {
            let length = read_length_encoded_int(data, marker)?;
            let mut result = Vec::with_capacity(length as usize);
            for _ in 0..length {
                result.push(data[*marker]);
                *marker += 1;
            }
            Some(String::from_utf8(result).unwrap().parse().unwrap())
        }
        0b10 => {
            *marker += 4; // Discard the remaining 6 bits. The next 4 bytes from the stream represent the length
            Some(u64::from_le_bytes(data[*marker - 4..*marker].try_into().unwrap()))
        }
        0b01 => {
            let octet1 = original & 0xFC;
            let octet2 = data[*marker];
            *marker += 1;
            Some(u16::from_le_bytes([octet1, octet2]) as u64)
        }
        0b00 => {
            let length = data[*marker];
            *marker += 1;
            if length == 0 {
                Some(0)
            } else {
                // Here we need to implement the rest of the next 6 bits representing the length
                let mut result = 0;
                for _ in 0..length {
                    result <<= 6;
                    result |= data[*marker] as u64;
                    *marker += 1;
                }
                Some(result)
            }
        }
        _ => panic!("Unreachable statement"),
    }
}

fn read_resizedb_field(data: &[u8], marker: &mut usize) -> bool {
    if data[*marker] != 0xFB {
        return false;
    }
    *marker += 1;
    let _ = read_length_encoded_int(data, marker);
    true
}

fn read_value(store: &mut Store, data: &[u8], marker: &mut usize) {
    let value_type = data[*marker];
    *marker += 1;
    if value_type != 0 {
        println!("Unknown key type: {}", value_type);
        return;
    }
    if let Some(key) = read_length_string(data, marker) {
        if let Some(value) = read_length_string(data, marker) {
            store.set(key, Entry::new(value, None));
        }
    }
}

fn read_length_string(data: &[u8], marker: &mut usize) -> Option<String> {
    let length = data[*marker] as usize;
    *marker += 1;
    let start = *marker;
    let end = start + length;
    if end > data.len() {
        return None;
    }
    let slice = &data[start..end];
    *marker += length;
    String::from_utf8(slice.to_vec()).ok()
}
