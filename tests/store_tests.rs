use medusa::store::{Store, Value, ValueWithTtl};
use std::thread;
use std::time::Duration;

#[test]
fn test_basic_set_get() {
    let store = Store::new();
    
    assert!(store.set("key1", "value1").is_ok());
    
    let result = store.get("key1").unwrap();
    assert_eq!(result, Some("value1".to_string()));
    
    let result = store.get("nonexistent").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_ttl_functionality() {
    let store = Store::new();
    
    assert!(store.set_with_ttl("ttl_key", "ttl_value", 1).is_ok());
    
    let result = store.get("ttl_key").unwrap();
    assert_eq!(result, Some("ttl_value".to_string()));
    
    let ttl = store.ttl("ttl_key").unwrap();
    assert!(ttl.is_some());
    assert!(ttl.unwrap() > 0);
    
    thread::sleep(Duration::from_millis(1100));
    
    let result = store.get("ttl_key").unwrap();
    assert_eq!(result, None);
    
    let ttl = store.ttl("ttl_key").unwrap();
    assert_eq!(ttl, Some(-1));
}

#[test]
fn test_expire_functionality() {
    let store = Store::new();
    
    assert!(store.set("expire_key", "expire_value").is_ok());
    
    let result = store.expire("expire_key", 1).unwrap();
    assert_eq!(result, true);
    
    let ttl = store.ttl("expire_key").unwrap();
    assert!(ttl.is_some());
    assert!(ttl.unwrap() > 0);
    
    thread::sleep(Duration::from_millis(1100));
    
    let result = store.get("expire_key").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_delete_functionality() {
    let store = Store::new();
    
    assert!(store.set("delete_key", "delete_value").is_ok());
    
    let result = store.delete("delete_key").unwrap();
    assert_eq!(result, Some("delete_value".to_string()));
    
    let result = store.get("delete_key").unwrap();
    assert_eq!(result, None);
    
    let result = store.delete("nonexistent").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_exists_functionality() {
    let store = Store::new();
    
    assert!(store.set("exists_key", "exists_value").is_ok());
    
    let result = store.exists("exists_key").unwrap();
    assert_eq!(result, true);
    
    let result = store.exists("nonexistent").unwrap();
    assert_eq!(result, false);
}

#[test]
fn test_list_keys_functionality() {
    let store = Store::new();
    
    assert!(store.set("key1", "value1").is_ok());
    assert!(store.set("key2", "value2").is_ok());
    assert!(store.set("key3", "value3").is_ok());
    
    let keys = store.list_keys().unwrap();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"key1".to_string()));
    assert!(keys.contains(&"key2".to_string()));
    assert!(keys.contains(&"key3".to_string()));
}

#[test]
fn test_count_functionality() {
    let store = Store::new();
    
    let count = store.count().unwrap();
    assert_eq!(count, 0);
    
    assert!(store.set("count_key1", "value1").is_ok());
    assert!(store.set("count_key2", "value2").is_ok());
    
    let count = store.count().unwrap();
    assert_eq!(count, 2);
    
    assert!(store.delete("count_key1").is_ok());
    
    let count = store.count().unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_expired_key_cleanup() {
    let store = Store::new();
    
    assert!(store.set_with_ttl("cleanup_key1", "value1", 1).is_ok());
    assert!(store.set_with_ttl("cleanup_key2", "value2", 1).is_ok());
    assert!(store.set("cleanup_key3", "value3").is_ok());
    
    let count = store.count().unwrap();
    assert_eq!(count, 3);
    
    thread::sleep(Duration::from_millis(1100));
    
    let count = store.count().unwrap();
    assert_eq!(count, 1);
    
    let keys = store.list_keys().unwrap();
    assert_eq!(keys.len(), 1);
    assert!(keys.contains(&"cleanup_key3".to_string()));
}

#[test]
fn test_concurrent_access() {
    let store = Store::new();
    let mut handles = vec![];
    
    for i in 0..10 {
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            let key = format!("concurrent_key_{}", i);
            let value = format!("value_{}", i);
            
            assert!(store_clone.set(&key, &value).is_ok());
            
            let result = store_clone.get(&key).unwrap();
            assert_eq!(result, Some(value));
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let count = store.count().unwrap();
    assert_eq!(count, 10);
}

#[test]
fn test_value_with_ttl_creation() {
    let value = Value::new("test".to_string());
    let value_with_ttl = ValueWithTtl::new(value);
    
    assert!(!value_with_ttl.is_expired());
    assert!(value_with_ttl.ttl_seconds().is_none());
    
    let value = Value::new("test".to_string());
    let value_with_ttl = ValueWithTtl::with_ttl(value, 5);
    
    assert!(!value_with_ttl.is_expired());
    let ttl = value_with_ttl.ttl_seconds().unwrap();
    assert!(ttl > 0 && ttl <= 5);
}