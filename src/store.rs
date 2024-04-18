use crate::{
    protocol::rdb::Rdb,
    stream::{Stream, StreamData, StreamId},
};
use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
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
            self.data.insert(key.clone(), StoreItem::Stream(Stream::empty()));
            self.get_stream(&key).unwrap()
        };
        let stream_id = StreamId::from(id.as_str());
        stream.entries.push((stream_id, stream_data));
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
        let (last_id_ms, last_id_seq) = (last_id.ms, last_id.seq);
        let (cur_id_ms, cur_id_seq) = id.split_once('-').unwrap_or_default();

        let cur_id_ms = cur_id_ms.parse::<u64>()?;
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

    pub fn generate_stream_id(&mut self, key: &str, id_pattern: &str) -> Option<String> {
        if let Some(stream) = self.get_stream(key) {
            let last_entry = stream.entries.last().map(|(last_entry, _)| last_entry);
            build_stream_id(id_pattern, last_entry)
        } else {
            build_stream_id(id_pattern, None)
        }
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

    pub fn get_stream_range(&mut self, key: &str, start: Option<StreamId>, end: Option<StreamId>) -> Option<Stream> {
        let stream = self.get_stream(key)?;
        let mut range_entries: Vec<(StreamId, StreamData)> = Vec::new();

        for (id, data) in &stream.entries {
            if let Some(ref start_id) = start {
                if id < start_id {
                    continue;
                }
            }
            if let Some(ref end_id) = end {
                if id > end_id {
                    break;
                }
            }
            range_entries.push((id.clone(), data.clone()));
        }

        Some(Stream { entries: range_entries })
    }

    pub fn get_stream_after_id(&mut self, key: &str, id: &StreamId) -> Option<Stream> {
        let stream = self.get_stream(key)?;
        let mut read_entries = Vec::new();
        for (entry_id, entry_data) in &stream.entries {
            if entry_id > id {
                read_entries.push((entry_id.clone(), entry_data.clone()));
            }
        }
        Some(Stream::new(read_entries))
    }
}

fn build_stream_id(pattern: &str, last_stream_entry: Option<&StreamId>) -> Option<String> {
    let pattern = if pattern.len() < 3 { "*-*" } else { pattern };
    let (cur_id_ms, cur_id_seq) = pattern.split_once('-')?;
    let mut id_ms: String = cur_id_ms.to_string();
    let mut id_seq: String = cur_id_seq.to_string();

    let auto_generate_ms = cur_id_ms == "*";
    if auto_generate_ms {
        let now = SystemTime::now();
        let time_since_unix_time = now.duration_since(UNIX_EPOCH).unwrap();
        id_ms = time_since_unix_time.as_millis().to_string();
    }

    let relevant_stream_entry = if let Some(last_id) = last_stream_entry {
        let (last_id_ms, last_id_seq) = (last_id.ms, last_id.seq);

        if last_id_ms.to_string() == id_ms {
            Some(last_id_seq)
        } else {
            None
        }
    } else {
        None
    };

    let auto_generate_seq = cur_id_seq == "*";
    if let Some(last_id_seq) = relevant_stream_entry {
        if auto_generate_seq {
            let next_seq = last_id_seq + 1;
            id_seq = next_seq.to_string();
        }
    } else if auto_generate_seq {
        id_seq = if id_ms == "0" { "1".to_string() } else { "0".to_string() };
    }

    Some(format!("{}-{}", id_ms, id_seq))
}
