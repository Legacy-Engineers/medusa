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
                let remaining = expires - now;
                // Return at least 1 second if there's any time remaining
                std::cmp::max(1, remaining.as_secs() as i64)
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
            Ok(map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
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
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(Some(-1))
                    } else {
                        Ok(value_with_ttl.ttl_seconds())
                    }
                } else {
                    // Check if this is a key that was recently accessed and found to be expired
                    // For now, we'll return None for truly non-existent keys
                    // But the test expects Some(-1) for expired keys
                    // We need a different approach - let's check if we just removed an expired key
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

    // Alias for keys_pattern for compatibility
    pub fn keys(&self, pattern: &str) -> Result<Vec<String>, String> {
        self.keys_pattern(pattern)
    }

    // Server info method
    pub fn info(&self) -> Result<String, String> {
        match self.map.lock() {
            Ok(mut map) => {
                map.retain(|_, value_with_ttl| !value_with_ttl.is_expired());
                let count = map.len();
                let info = format!(
                    "# Server\nmedusa_version:0.1.0\nuptime_in_seconds:unknown\n\n# Memory\nused_memory:{}\ntotal_keys:{}\n\n# Stats\ntotal_connections_received:unknown\ntotal_commands_processed:unknown",
                    count * 64, // rough estimate
                    count
                );
                Ok(info)
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    // Hash operations
    pub fn hset(&self, key: &str, field: &str, value: &str) -> Result<bool, String> {
        match self.map.lock() {
            Ok(mut map) => {
                let entry = map.entry(key.to_string()).or_insert_with(|| ValueWithTtl::new(Value::new_hash()));
                
                match &mut entry.value {
                    Value::Hash(ref mut hash) => {
                        let is_new = !hash.contains_key(field);
                        hash.insert(field.to_string(), value.to_string());
                        Ok(is_new)
                    }
                    _ => {
                        // Convert to hash if not already
                        let mut hash = HashMap::new();
                        hash.insert(field.to_string(), value.to_string());
                        entry.value = Value::Hash(hash);
                        Ok(true)
                    }
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn hget(&self, key: &str, field: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(None)
                    } else {
                        match &value_with_ttl.value {
                            Value::Hash(hash) => Ok(hash.get(field).cloned()),
                            _ => Err("Key contains non-hash value".to_string()),
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn hgetall(&self, key: &str) -> Result<HashMap<String, String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(HashMap::new())
                    } else {
                        match &value_with_ttl.value {
                            Value::Hash(hash) => Ok(hash.clone()),
                            _ => Err("Key contains non-hash value".to_string()),
                        }
                    }
                } else {
                    Ok(HashMap::new())
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn hdel(&self, key: &str, field: &str) -> Result<bool, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get_mut(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(false)
                    } else {
                        match &mut value_with_ttl.value {
                            Value::Hash(ref mut hash) => {
                                Ok(hash.remove(field).is_some())
                            }
                            _ => Err("Key contains non-hash value".to_string()),
                        }
                    }
                } else {
                    Ok(false)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn hexists(&self, key: &str, field: &str) -> Result<bool, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(false)
                    } else {
                        match &value_with_ttl.value {
                            Value::Hash(hash) => Ok(hash.contains_key(field)),
                            _ => Err("Key contains non-hash value".to_string()),
                        }
                    }
                } else {
                    Ok(false)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn hlen(&self, key: &str) -> Result<usize, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(0)
                    } else {
                        match &value_with_ttl.value {
                            Value::Hash(hash) => Ok(hash.len()),
                            _ => Err("Key contains non-hash value".to_string()),
                        }
                    }
                } else {
                    Ok(0)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    // List operations
    pub fn lpush(&self, key: &str, value: &str) -> Result<usize, String> {
        match self.map.lock() {
            Ok(mut map) => {
                let entry = map.entry(key.to_string()).or_insert_with(|| ValueWithTtl::new(Value::new_list()));
                
                match &mut entry.value {
                    Value::List(ref mut list) => {
                        list.push_front(value.to_string());
                        Ok(list.len())
                    }
                    _ => {
                        // Convert to list if not already
                        let mut list = VecDeque::new();
                        list.push_front(value.to_string());
                        entry.value = Value::List(list);
                        Ok(1)
                    }
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn rpush(&self, key: &str, value: &str) -> Result<usize, String> {
        match self.map.lock() {
            Ok(mut map) => {
                let entry = map.entry(key.to_string()).or_insert_with(|| ValueWithTtl::new(Value::new_list()));
                
                match &mut entry.value {
                    Value::List(ref mut list) => {
                        list.push_back(value.to_string());
                        Ok(list.len())
                    }
                    _ => {
                        // Convert to list if not already
                        let mut list = VecDeque::new();
                        list.push_back(value.to_string());
                        entry.value = Value::List(list);
                        Ok(1)
                    }
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn lpop(&self, key: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get_mut(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(None)
                    } else {
                        match &mut value_with_ttl.value {
                            Value::List(ref mut list) => Ok(list.pop_front()),
                            _ => Err("Key contains non-list value".to_string()),
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn rpop(&self, key: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get_mut(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(None)
                    } else {
                        match &mut value_with_ttl.value {
                            Value::List(ref mut list) => Ok(list.pop_back()),
                            _ => Err("Key contains non-list value".to_string()),
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn llen(&self, key: &str) -> Result<usize, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(0)
                    } else {
                        match &value_with_ttl.value {
                            Value::List(list) => Ok(list.len()),
                            _ => Err("Key contains non-list value".to_string()),
                        }
                    }
                } else {
                    Ok(0)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value_with_ttl) = map.get(key) {
                    if value_with_ttl.is_expired() {
                        map.remove(key);
                        Ok(Vec::new())
                    } else {
                        match &value_with_ttl.value {
                            Value::List(list) => {
                                let len = list.len() as i64;
                                if len == 0 {
                                    return Ok(Vec::new());
                                }

                                // Handle negative indices
                                let start_idx = if start < 0 { 
                                    std::cmp::max(0, len + start) as usize 
                                } else { 
                                    std::cmp::min(start as usize, len as usize) 
                                };
                                
                                let stop_idx = if stop < 0 { 
                                    std::cmp::max(0, len + stop) as usize 
                                } else { 
                                    std::cmp::min(stop as usize, len as usize - 1) 
                                };

                                if start_idx > stop_idx {
                                    return Ok(Vec::new());
                                }

                                let result: Vec<String> = list.iter()
                                    .skip(start_idx)
                                    .take(stop_idx - start_idx + 1)
                                    .cloned()
                                    .collect();
                                
                                Ok(result)
                            }
                            _ => Err("Key contains non-list value".to_string()),
                        }
                    }
                } else {
                    Ok(Vec::new())
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }
}
