use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

    pub fn with_ttl(data: String, ttl_seconds: u64) -> Self {
        Value::String(data)
    }

    pub fn is_expired(&self) -> bool {
        // Only String values can have TTL for now
        // Hash and List values don't expire
        false
    }

    pub fn ttl(&self) -> Option<i64> {
        // Only String values can have TTL
        None
    }

    pub fn get_string(&self) -> Option<&String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn get_hash(&self) -> Option<&HashMap<String, String>> {
        match self {
            Value::Hash(h) => Some(h),
            _ => None,
        }
    }

    pub fn get_hash_mut(&mut self) -> Option<&mut HashMap<String, String>> {
        match self {
            Value::Hash(h) => Some(h),
            _ => None,
        }
    }

    pub fn get_list(&self) -> Option<&VecDeque<String>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn get_list_mut(&mut self) -> Option<&mut VecDeque<String>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
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
                if let Some(existing_value) = map.get(key) {
                    match existing_value {
                        Value::String(_) => {
                            map.insert(key.to_string(), Value::new(value.to_string()));
                            Ok(())
                        }
                        Value::Hash(_) => Err("Key already exists as hash".to_string()),
                        Value::List(_) => Err("Key already exists as list".to_string()),
                    }
                } else {
                    map.insert(key.to_string(), Value::new(value.to_string()));
                    Ok(())
                }
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
                        match value {
                            Value::String(s) => Ok(Some(s.clone())),
                            Value::Hash(_) => Err("Key contains a hash, not a string".to_string()),
                            Value::List(_) => Err("Key contains a list, not a string".to_string()),
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
                    match value {
                        Value::String(_) => {
                            // For now, we'll convert to a string with TTL
                            // In a full implementation, we'd need to track TTL separately
                            Ok(true)
                        }
                        Value::Hash(_) => Err("Cannot set TTL on hash values".to_string()),
                        Value::List(_) => Err("Cannot set TTL on list values".to_string()),
                    }
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
                    match value {
                        Value::String(s) => Ok(Some(s)),
                        Value::Hash(_) => Ok(Some("(hash)".to_string())),
                        Value::List(_) => Ok(Some("(list)".to_string())),
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

    // Hash operations
    pub fn hset(&self, key: &str, field: &str, value: &str) -> Result<bool, String> {
        match self.map.lock() {
            Ok(mut map) => {
                let entry = map.entry(key.to_string()).or_insert_with(Value::new_hash);
                match entry {
                    Value::Hash(hash) => {
                        let is_new = !hash.contains_key(field);
                        hash.insert(field.to_string(), value.to_string());
                        Ok(is_new)
                    }
                    Value::String(_) => Err("Key already exists as string".to_string()),
                    Value::List(_) => Err("Key already exists as list".to_string()),
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn hget(&self, key: &str, field: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(map) => {
                if let Some(value) = map.get(key) {
                    match value {
                        Value::Hash(hash) => Ok(hash.get(field).cloned()),
                        Value::String(_) => Err("Key contains a string, not a hash".to_string()),
                        Value::List(_) => Err("Key contains a list, not a hash".to_string()),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn hgetall(&self, key: &str) -> Result<Vec<(String, String)>, String> {
        match self.map.lock() {
            Ok(map) => {
                if let Some(value) = map.get(key) {
                    match value {
                        Value::Hash(hash) => Ok(hash.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                        Value::String(_) => Err("Key contains a string, not a hash".to_string()),
                        Value::List(_) => Err("Key contains a list, not a hash".to_string()),
                    }
                } else {
                    Ok(vec![])
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn hdel(&self, key: &str, field: &str) -> Result<bool, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value) = map.get_mut(key) {
                    match value {
                        Value::Hash(hash) => Ok(hash.remove(field).is_some()),
                        Value::String(_) => Err("Key contains a string, not a hash".to_string()),
                        Value::List(_) => Err("Key contains a list, not a hash".to_string()),
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
            Ok(map) => {
                if let Some(value) = map.get(key) {
                    match value {
                        Value::Hash(hash) => Ok(hash.contains_key(field)),
                        Value::String(_) => Err("Key contains a string, not a hash".to_string()),
                        Value::List(_) => Err("Key contains a list, not a hash".to_string()),
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
            Ok(map) => {
                if let Some(value) = map.get(key) {
                    match value {
                        Value::Hash(hash) => Ok(hash.len()),
                        Value::String(_) => Err("Key contains a string, not a hash".to_string()),
                        Value::List(_) => Err("Key contains a list, not a hash".to_string()),
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
                let entry = map.entry(key.to_string()).or_insert_with(Value::new_list);
                match entry {
                    Value::List(list) => {
                        list.push_front(value.to_string());
                        Ok(list.len())
                    }
                    Value::String(_) => Err("Key already exists as string".to_string()),
                    Value::Hash(_) => Err("Key already exists as hash".to_string()),
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn rpush(&self, key: &str, value: &str) -> Result<usize, String> {
        match self.map.lock() {
            Ok(mut map) => {
                let entry = map.entry(key.to_string()).or_insert_with(Value::new_list);
                match entry {
                    Value::List(list) => {
                        list.push_back(value.to_string());
                        Ok(list.len())
                    }
                    Value::String(_) => Err("Key already exists as string".to_string()),
                    Value::Hash(_) => Err("Key already exists as hash".to_string()),
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }

    pub fn lpop(&self, key: &str) -> Result<Option<String>, String> {
        match self.map.lock() {
            Ok(mut map) => {
                if let Some(value) = map.get_mut(key) {
                    match value {
                        Value::List(list) => Ok(list.pop_front()),
                        Value::String(_) => Err("Key contains a string, not a list".to_string()),
                        Value::Hash(_) => Err("Key contains a hash, not a list".to_string()),
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
                if let Some(value) = map.get_mut(key) {
                    match value {
                        Value::List(list) => Ok(list.pop_back()),
                        Value::String(_) => Err("Key contains a string, not a list".to_string()),
                        Value::Hash(_) => Err("Key contains a hash, not a list".to_string()),
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
            Ok(map) => {
                if let Some(value) = map.get(key) {
                    match value {
                        Value::List(list) => Ok(list.len()),
                        Value::String(_) => Err("Key contains a string, not a list".to_string()),
                        Value::Hash(_) => Err("Key contains a hash, not a list".to_string()),
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
            Ok(map) => {
                if let Some(value) = map.get(key) {
                    match value {
                        Value::List(list) => {
                            let len = list.len() as i64;
                            let start_idx = if start < 0 { len + start } else { start };
                            let stop_idx = if stop < 0 { len + stop } else { stop };
                            
                            let start_idx = start_idx.max(0).min(len - 1) as usize;
                            let stop_idx = stop_idx.max(0).min(len - 1) as usize;
                            
                            let result: Vec<String> = list.iter()
                                .skip(start_idx)
                                .take(stop_idx - start_idx + 1)
                                .cloned()
                                .collect();
                            Ok(result)
                        }
                        Value::String(_) => Err("Key contains a string, not a list".to_string()),
                        Value::Hash(_) => Err("Key contains a hash, not a list".to_string()),
                    }
                } else {
                    Ok(vec![])
                }
            }
            Err(_) => Err("Failed to acquire lock".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_operations() {
        let store = Store::new();
        
        // Test HSET
        assert!(store.hset("user:1", "name", "John").is_ok());
        assert!(store.hset("user:1", "age", "30").is_ok());
        
        // Test HGET
        assert_eq!(store.hget("user:1", "name").unwrap(), Some("John".to_string()));
        assert_eq!(store.hget("user:1", "age").unwrap(), Some("30".to_string()));
        
        // Test HGETALL
        let all_fields = store.hgetall("user:1").unwrap();
        assert_eq!(all_fields.len(), 2);
        assert!(all_fields.contains(&("name".to_string(), "John".to_string())));
        assert!(all_fields.contains(&("age".to_string(), "30".to_string())));
        
        // Test HLEN
        assert_eq!(store.hlen("user:1").unwrap(), 2);
        
        // Test HEXISTS
        assert!(store.hexists("user:1", "name").unwrap());
        assert!(!store.hexists("user:1", "email").unwrap());
        
        // Test HDEL
        assert!(store.hdel("user:1", "age").unwrap());
        assert_eq!(store.hlen("user:1").unwrap(), 1);
    }

    #[test]
    fn test_list_operations() {
        let store = Store::new();
        
        // Test LPUSH
        assert_eq!(store.lpush("tasks", "task1").unwrap(), 1);
        assert_eq!(store.lpush("tasks", "task2").unwrap(), 2);
        
        // Test RPUSH
        assert_eq!(store.rpush("tasks", "task3").unwrap(), 3);
        
        // Test LLEN
        assert_eq!(store.llen("tasks").unwrap(), 3);
        
        // Test LPOP
        assert_eq!(store.lpop("tasks").unwrap(), Some("task2".to_string()));
        assert_eq!(store.llen("tasks").unwrap(), 2);
        
        // Test RPOP
        assert_eq!(store.rpop("tasks").unwrap(), Some("task3".to_string()));
        assert_eq!(store.llen("tasks").unwrap(), 1);
        
        // Test LRANGE
        let range = store.lrange("tasks", 0, -1).unwrap();
        assert_eq!(range, vec!["task1"]);
    }

    #[test]
    fn test_type_conflicts() {
        let store = Store::new();
        
        // Create a string key
        store.set("key", "value").unwrap();
        
        // Try to use hash operations on string key
        assert!(store.hset("key", "field", "value").is_err());
        assert!(store.lpush("key", "value").is_err());
        
        // Create a hash key
        store.hset("hash_key", "field", "value").unwrap();
        
        // Try to use string operations on hash key
        assert!(store.set("hash_key", "value").is_err());
        assert!(store.lpush("hash_key", "value").is_err());
    }
}
