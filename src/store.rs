use crate::{
    protocol::rdb::Rdb,
    stream::{Stream, StreamData},
};
use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

pub trait EntryValue {
    fn value_type(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub value: String,
    pub expiry_time: Option<Duration>,
    pub expiry_at: Option<SystemTime>,
}

impl EntryValue for Entry {
    fn value_type(&self) -> String {
        "string".to_string()
    }
}

impl Entry {
    pub fn new(value: String, expiry: Option<Duration>) -> Self {
        if expiry.is_some() {
            let current_time = SystemTime::now();
            let expiry_time = current_time + expiry.unwrap();
            Self {
                value,
                expiry_time: expiry,
                expiry_at: Some(expiry_time),
            }
        } else {
            Self {
                value,
                expiry_at: None,
                expiry_time: None,
            }
        }
    }
}

#[derive(Debug)]
pub enum StoreItem {
    KeyValueEntry(Entry),
    Stream(Stream),
}

impl EntryValue for StoreItem {
    fn value_type(&self) -> String {
        match self {
            Self::KeyValueEntry(x) => x.value_type(),
            Self::Stream(_) => "stream".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Store {
    pub data: HashMap<String, StoreItem>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }
    pub fn set_kv(&mut self, key: String, entry: Entry) -> Result<()> {
        self.data.insert(key, StoreItem::KeyValueEntry(entry));
        Ok(())
    }

    pub fn get_kv(&self, key: &str) -> Option<&Entry> {
        let store_item = self.data.get(key)?;
        let entry = if let StoreItem::KeyValueEntry(e) = store_item {
            e
        } else {
            return None;
        };

        if let Some(expiry_date_time) = entry.expiry_at {
            if SystemTime::now() > expiry_date_time {
                return None;
            }
        }
        Some(entry)
    }

    pub fn get_store_item(&self, key: &str) -> Option<&StoreItem> {
        self.data.get(key)
    }

    pub fn get_stream(&mut self, key: &str) -> Option<&mut Stream> {
        let item = self.data.get_mut(key)?;
        if let StoreItem::Stream(stream) = item {
            Some(stream)
        } else {
            None
        }
    }

    pub fn set_stream(&mut self, key: String, id: String, stream_data: StreamData) -> Result<()> {
        let stream = if let Some(stream) = self.get_stream(&key) {
            stream
        } else {
            self.data.insert(key.clone(), StoreItem::Stream(Stream::new()));
            self.get_stream(&key).unwrap()
        };
        stream.entries.push((id, stream_data));
        Ok(())
    }

    pub fn validate_stream(&mut self, key: &str, id: &str) -> Result<()> {
        let stream = match self.get_stream(key) {
            Some(stream) => stream,
            None => return Ok(()),
        };

        if stream.entries.is_empty() {
            return Ok(());
        }

        let (last_id, _) = stream.entries.last().unwrap();
        let (last_id_ms, last_id_seq) = last_id.split_once('-').unwrap_or_default();
        let (cur_id_ms, cur_id_seq) = id.split_once('-').unwrap_or_default();

        let last_id_ms = last_id_ms.parse::<u64>()?;
        let cur_id_ms = cur_id_ms.parse::<u64>()?;
        let last_id_seq = last_id_seq.parse::<u64>()?;
        let cur_id_seq = cur_id_seq.parse::<u64>()?;

        if cur_id_ms == 0 && cur_id_seq == 0 {
            return Err(anyhow!("ERR The ID specified in XADD must be greater than 0-0"));
        }

        if cur_id_ms < last_id_ms || (cur_id_ms == last_id_ms && cur_id_seq <= last_id_seq) {
            return Err(anyhow!(
                "ERR The ID specified in XADD is equal or smaller than the target stream top item"
            ));
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn import_rdb(&mut self, data: &[u8]) -> Result<()> {
        Rdb::parse_rdb(self, data)
    }
}
