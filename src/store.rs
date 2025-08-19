use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct ValueWithTtl {
    pub value: Value,
    pub expires_at: Option<Instant>,
}

impl ValueWithTtl {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            expires_at: None,
        }
    }

    pub fn with_ttl(value: Value, ttl_seconds: u64) -> Self {
        Self {
            value,
            expires_at: Some(Instant::now() + Duration::from_secs(ttl_seconds)),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |expires| Instant::now() > expires)
    }

    pub fn ttl_seconds(&self) -> Option<i64> {
        self.expires_at.map(|expires| {
            let now = Instant::now();
            if now > expires {
                -1
            } else {
                (expires - now).as_secs() as i64
            }
        })
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Hash(HashMap<String, String>),
    List(VecDeque<String>),
}

impl Value {
    pub fn new(data: String) -> Self {
        Value::String(data)
    }

    pub fn new_hash() -> Self {
        Value::Hash(HashMap::new())
    }

    pub fn new_list() -> Self {
        Value::List(VecDeque::new())
    }
}

#[derive(Clone)]
pub struct Store {
    map: Arc<Mutex<HashMap<String, ValueWithTtl>>>,
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
                map.insert(key.to_string(), ValueWithTtl::new(Value::new(value.to_string())));
                Ok(())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn set_with_ttl(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), String> {
        match self.map.lock() {
            Ok(mut map) => {
                map.insert(key.to_string(), ValueWithTtl::with_ttl(Value::new(value.to_string()), ttl_seconds));
                Ok(())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(None)
                    } else {
                        match &value_with_ttl.value {
                            Value::String(s) => Ok(Some(s.clone())),
                            _ => Err("Key contains non-string value".to_string()),
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn ttl(&self, key: &str) -> Result<Option<i64>, String> {
        match self.map.lock() {
            Ok(map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        Ok(Some(-1))
                    } else {
                        Ok(value_with_ttl.ttl_seconds())
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn expire(&self, key: &str, ttl_seconds: u64) -> Result<bool, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get_mut(key) {
                    value_with_ttl.expires_at = Some(Instant::now() + Duration::from_secs(ttl_seconds));
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn delete(&self, key: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.remove(key) {
                    match value_with_ttl.value {
                        Value::String(s) => Ok(Some(s)),
                        _ => Ok(Some("(non-string)".to_string())),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn list_keys(&self) -> Result<Vec<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                map.retain(|_, value_with_ttl| !value_with_ttl.is_expired());
                Ok(map.keys().cloned().collect())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn count(&self) -> Result<usize, String> {
        match self.map.lock() {
            Ok(mut map) => {
                map.retain(|_, value_with_ttl| !value_with_ttl.is_expired());
                Ok(map.len())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn exists(&self, key: &str) -> Result<bool, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(false)
                    } else {
                        Ok(true)
                    }
                } else {
                    Ok(false)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn keys_pattern(&self, pattern: &str) -> Result<Vec<String>, String> {
        let keys = self.list_keys()?;
        if pattern == "*" {
            return Ok(keys);
        }
        
        Ok(keys.into_iter().filter(|key| {
            if pattern.contains('*') {
                let prefix = pattern.split('*').next().unwrap_or("");
                key.starts_with(prefix)
            } else {
                key == pattern
            }
        }).collect())
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

    pub fn flush_all(&self) -> Result<(), String> {
        self.clear()
    }
}
