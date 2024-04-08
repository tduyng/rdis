use std::collections::HashMap;

use crate::utils::current_time_ms;

pub struct Database {
    data: HashMap<String, (String, u128)>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, (value, 0));
    }

    pub fn set_with_expiry(&mut self, key: String, value: String, expiry_ms: u128) {
        self.data.insert(key, (value, expiry_ms));
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        match self.data.get(key) {
            Some((value, expiry)) if *expiry == 0 || *expiry > current_time_ms() => Some(value),
            _ => None,
        }
    }
}
