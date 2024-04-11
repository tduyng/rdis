use tokio::net::TcpStream;

use crate::utils::current_time_ms;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct RedisStore {
    pub data: Arc<Mutex<HashMap<String, (String, u128)>>>,
    pub repl_streams: Vec<TcpStream>,
}

impl Default for RedisStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RedisStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            repl_streams: Vec::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.lock().unwrap().insert(key, (value, 0));
    }

    pub fn set_with_expiry(&mut self, key: String, value: String, expiry_ms: u128) {
        self.data.lock().unwrap().insert(key, (value, expiry_ms));
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let store = self.data.lock().unwrap();
        let (value, expiry) = store.get(key).unwrap();
        let value = value.clone();
        if *expiry == 0 || *expiry > current_time_ms() {
            return Some(value);
        }
        None
    }

    pub fn add_repl_streams(&mut self, stream: TcpStream) {
        self.repl_streams.push(stream)
    }
}
