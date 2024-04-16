use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

#[derive(Debug, Clone)]
pub struct Entry {
    pub value: String,
    pub expiry_time: Option<Duration>,
    pub expiry_at: Option<SystemTime>,
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
                expiry_time: None,
                expiry_at: None,
            }
        }
    }
}

#[derive(Debug)]
pub struct Store {
    pub data: HashMap<String, Entry>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn set(&mut self, key: String, entry: Entry) {
        self.data.insert(key, entry);
    }

    pub fn get(&self, key: String) -> Option<&Entry> {
        let entry = self.data.get(&key);
        entry?;
        let entry = entry.unwrap();
        if let Some(expiry_date_time) = entry.expiry_at {
            if SystemTime::now() > expiry_date_time {
                return None;
            }
        }
        Some(entry)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
