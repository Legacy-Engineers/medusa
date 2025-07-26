use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Value {
    pub data: String,
    pub expires_at: Option<Instant>,
}

impl Value {
    pub fn new(data: String) -> Self {
        Value {
            data,
            expires_at: None,
        }
    }

    pub fn with_ttl(data: String, ttl_seconds: u64) -> Self {
        Value {
            data,
            expires_at: Some(Instant::now() + Duration::from_secs(ttl_seconds)),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() >= expires_at
        } else {
            false
        }
    }

    pub fn ttl(&self) -> Option<i64> {
        self.expires_at.map(|expires_at| {
            let remaining = expires_at.duration_since(Instant::now());
            if remaining.is_zero() {
                -1 // Expired
            } else {
                remaining.as_secs() as i64
            }
        })
    }
}

#[derive(Clone)]
pub struct Store {
    map: Arc<Mutex<HashMap<String, Value>>>,
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
                map.insert(key.to_string(), Value::new(value.to_string()));
                Ok(())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn set_with_ttl(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), String> {
        match self.map.lock() {
            Ok(mut map) => {
                map.insert(key.to_string(), Value::with_ttl(value.to_string(), ttl_seconds));
                Ok(())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value) = map.get(key) {
                    if value.is_expired() {
                        map.remove(key);
                        Ok(None)
                    } else {
                        Ok(Some(value.data.clone()))
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
                if let Some(value) = map.get(key) {
                    if value.is_expired() {
                        Ok(Some(-1))
                    } else {
                        Ok(value.ttl())
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
                if let Some(value) = map.get_mut(key) {
                    value.expires_at = Some(Instant::now() + Duration::from_secs(ttl_seconds));
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
                if let Some(value) = map.remove(key) {
                    Ok(Some(value.data))
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
                // Clean up expired keys
                map.retain(|_, value| !value.is_expired());
                Ok(map.keys().cloned().collect())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn count(&self) -> Result<usize, String> {
        match self.map.lock() {
            Ok(mut map) => {
                // Clean up expired keys
                map.retain(|_, value| !value.is_expired());
                Ok(map.len())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn exists(&self, key: &str) -> Result<bool, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value) = map.get(key) {
                    if value.is_expired() {
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

    pub fn clear(&self) -> Result<(), String> {
        match self.map.lock() {
            Ok(mut map) => {
                map.clear();
                Ok(())
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn keys(&self, pattern: &str) -> Result<Vec<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                // Clean up expired keys
                map.retain(|_, value| !value.is_expired());
                
                let keys: Vec<String> = map.keys()
                    .filter(|key| self.matches_pattern(key, pattern))
                    .cloned()
                    .collect();
                Ok(keys)
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    fn matches_pattern(&self, key: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        // Simple pattern matching (can be enhanced)
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return key.starts_with(prefix) && key.ends_with(suffix);
            }
        }
        
        key == pattern
    }

    pub fn flush_all(&self) -> Result<(), String> {
        self.clear()
    }

    pub fn info(&self) -> Result<String, String> {
        match self.map.lock() {
            Ok(map) => {
                let total_keys = map.len();
                let expired_keys = map.values().filter(|v| v.is_expired()).count();
                let active_keys = total_keys - expired_keys;
                
                Ok(format!(
                    "Total keys: {}\nActive keys: {}\nExpired keys: {}",
                    total_keys, active_keys, expired_keys
                ))
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }
}
