use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Store {
    map: Arc<Mutex<HashMap<String, String>>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), String> {
        match self.map.lock() {
            Ok(mut map) => {
                map.insert(key.to_string(), value.to_string());
                Ok(())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(map) => Ok(map.get(key).cloned()),
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn delete(&self, key: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(mut map) => Ok(map.remove(key)),
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn list_keys(&self) -> Result<Vec<String>, String> {
        match self.map.lock() {
            Ok(map) => Ok(map.keys().cloned().collect()),
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn count(&self) -> Result<usize, String> {
        match self.map.lock() {
            Ok(map) => Ok(map.len()),
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn exists(&self, key: &str) -> Result<bool, String> {
        match self.map.lock() {
            Ok(map) => Ok(map.contains_key(key)),
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn clear(&self) -> Result<(), String> {
        match self.map.lock() {
            Ok(mut map) => {
                map.clear();
                Ok(())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }
}
