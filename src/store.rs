use crate::utils::current_time_ms;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Database {
    data: HashMap<String, (String, u128)>,
}

lazy_static! {
    pub static ref DATABASE: Mutex<Option<Database>> = Mutex::new(Some(Database::new()));
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub async fn instance() -> Database {
        if let Some(instance) = DATABASE.lock().await.clone() {
            instance
        } else {
            let new_instance = Database::new();
            *DATABASE.lock().await = Some(new_instance.clone());
            new_instance
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
