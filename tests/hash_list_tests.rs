use medusa::store::Store;
use std::thread;
use std::time::Duration;

#[test]
fn test_hash_operations() {
    let store = Store::new();
    
    // Test HSET and HGET
    assert!(store.hset("user:1", "name", "John").unwrap());
    assert!(!store.hset("user:1", "name", "Johnny").unwrap()); // Should return false for update
    assert!(store.hset("user:1", "age", "30").unwrap());
    
    assert_eq!(store.hget("user:1", "name").unwrap(), Some("Johnny".to_string()));
    assert_eq!(store.hget("user:1", "age").unwrap(), Some("30".to_string()));
    assert_eq!(store.hget("user:1", "nonexistent").unwrap(), None);
    assert_eq!(store.hget("nonexistent", "field").unwrap(), None);
    
    // Test HGETALL
    let all_fields = store.hgetall("user:1").unwrap();
    assert_eq!(all_fields.len(), 2);
    assert_eq!(all_fields.get("name"), Some(&"Johnny".to_string()));
    assert_eq!(all_fields.get("age"), Some(&"30".to_string()));
    
    let empty_hash = store.hgetall("nonexistent").unwrap();
    assert!(empty_hash.is_empty());
    
    // Test HEXISTS
    assert!(store.hexists("user:1", "name").unwrap());
    assert!(store.hexists("user:1", "age").unwrap());
    assert!(!store.hexists("user:1", "nonexistent").unwrap());
    assert!(!store.hexists("nonexistent", "field").unwrap());
    
    // Test HLEN
    assert_eq!(store.hlen("user:1").unwrap(), 2);
    assert_eq!(store.hlen("nonexistent").unwrap(), 0);
    
    // Test HDEL
    assert!(store.hdel("user:1", "age").unwrap());
    assert!(!store.hdel("user:1", "age").unwrap()); // Should return false for non-existent field
    assert!(!store.hdel("nonexistent", "field").unwrap());
    
    assert_eq!(store.hlen("user:1").unwrap(), 1);
    assert!(!store.hexists("user:1", "age").unwrap());
    assert!(store.hexists("user:1", "name").unwrap());
}

#[test]
fn test_list_operations() {
    let store = Store::new();
    
    // Test LPUSH and RPUSH
    assert_eq!(store.lpush("mylist", "first").unwrap(), 1);
    assert_eq!(store.rpush("mylist", "last").unwrap(), 2);
    assert_eq!(store.lpush("mylist", "very_first").unwrap(), 3);
    
    // Test LLEN
    assert_eq!(store.llen("mylist").unwrap(), 3);
    assert_eq!(store.llen("nonexistent").unwrap(), 0);
    
    // Test LRANGE
    let full_range = store.lrange("mylist", 0, -1).unwrap();
    assert_eq!(full_range, vec!["very_first", "first", "last"]);
    
    let partial_range = store.lrange("mylist", 0, 1).unwrap();
    assert_eq!(partial_range, vec!["very_first", "first"]);
    
    let negative_range = store.lrange("mylist", -2, -1).unwrap();
    assert_eq!(negative_range, vec!["first", "last"]);
    
    let empty_range = store.lrange("nonexistent", 0, -1).unwrap();
    assert!(empty_range.is_empty());
    
    // Test LPOP and RPOP
    assert_eq!(store.lpop("mylist").unwrap(), Some("very_first".to_string()));
    assert_eq!(store.llen("mylist").unwrap(), 2);
    
    assert_eq!(store.rpop("mylist").unwrap(), Some("last".to_string()));
    assert_eq!(store.llen("mylist").unwrap(), 1);
    
    assert_eq!(store.lpop("mylist").unwrap(), Some("first".to_string()));
    assert_eq!(store.llen("mylist").unwrap(), 0);
    
    // Test operations on empty list
    assert_eq!(store.lpop("mylist").unwrap(), None);
    assert_eq!(store.rpop("mylist").unwrap(), None);
    assert_eq!(store.lpop("nonexistent").unwrap(), None);
    assert_eq!(store.rpop("nonexistent").unwrap(), None);
}

#[test]
fn test_hash_with_ttl() {
    let store = Store::new();
    
    // Create hash with TTL
    assert!(store.hset("temp_user", "name", "Alice").unwrap());
    assert!(store.hset("temp_user", "email", "alice@example.com").unwrap());
    assert!(store.expire("temp_user", 1).unwrap());
    
    // Should exist initially
    assert!(store.hexists("temp_user", "name").unwrap());
    assert_eq!(store.hlen("temp_user").unwrap(), 2);
    
    // Wait for expiration
    thread::sleep(Duration::from_millis(1100));
    
    // Should be expired
    assert!(!store.hexists("temp_user", "name").unwrap());
    assert_eq!(store.hlen("temp_user").unwrap(), 0);
    assert!(store.hgetall("temp_user").unwrap().is_empty());
}

#[test]
fn test_list_with_ttl() {
    let store = Store::new();
    
    // Create list with TTL
    assert_eq!(store.lpush("temp_list", "item1").unwrap(), 1);
    assert_eq!(store.lpush("temp_list", "item2").unwrap(), 2);
    assert!(store.expire("temp_list", 1).unwrap());
    
    // Should exist initially
    assert_eq!(store.llen("temp_list").unwrap(), 2);
    
    // Wait for expiration
    thread::sleep(Duration::from_millis(1100));
    
    // Should be expired
    assert_eq!(store.llen("temp_list").unwrap(), 0);
    assert!(store.lrange("temp_list", 0, -1).unwrap().is_empty());
    assert_eq!(store.lpop("temp_list").unwrap(), None);
}

#[test]
fn test_keys_pattern_matching() {
    let store = Store::new();
    
    // Set up test data
    assert!(store.set("user:1", "john").is_ok());
    assert!(store.set("user:2", "jane").is_ok());
    assert!(store.set("product:1", "laptop").is_ok());
    assert!(store.set("product:2", "mouse").is_ok());
    assert!(store.set("config:debug", "true").is_ok());
    
    // Test pattern matching
    let user_keys = store.keys("user:*").unwrap();
    assert_eq!(user_keys.len(), 2);
    assert!(user_keys.contains(&"user:1".to_string()));
    assert!(user_keys.contains(&"user:2".to_string()));
    
    let product_keys = store.keys("product:*").unwrap();
    assert_eq!(product_keys.len(), 2);
    assert!(product_keys.contains(&"product:1".to_string()));
    assert!(product_keys.contains(&"product:2".to_string()));
    
    let config_keys = store.keys("config:*").unwrap();
    assert_eq!(config_keys.len(), 1);
    assert!(config_keys.contains(&"config:debug".to_string()));
    
    // Test wildcard
    let all_keys = store.keys("*").unwrap();
    assert_eq!(all_keys.len(), 5);
    
    // Test non-matching pattern
    let no_keys = store.keys("nonexistent:*").unwrap();
    assert!(no_keys.is_empty());
}

#[test]
fn test_info_command() {
    let store = Store::new();
    
    // Add some data
    assert!(store.set("key1", "value1").is_ok());
    assert!(store.hset("hash1", "field1", "value1").unwrap());
    assert_eq!(store.lpush("list1", "item1").unwrap(), 1);
    
    let info = store.info().unwrap();
    assert!(info.contains("medusa_version:0.1.0"));
    assert!(info.contains("total_keys:3"));
    assert!(info.contains("# Server"));
    assert!(info.contains("# Memory"));
    assert!(info.contains("# Stats"));
}

#[test]
fn test_type_conversion() {
    let store = Store::new();
    
    // Start with a string
    assert!(store.set("convertible", "original").is_ok());
    assert_eq!(store.get("convertible").unwrap(), Some("original".to_string()));
    
    // Convert to hash by using hset
    assert!(store.hset("convertible", "field", "value").unwrap());
    assert_eq!(store.hget("convertible", "field").unwrap(), Some("value".to_string()));
    
    // Should error when trying to get as string
    assert!(store.get("convertible").is_err());
    
    // Start fresh with another key for list conversion
    assert!(store.set("convertible2", "original").is_ok());
    
    // Convert to list by using lpush
    assert_eq!(store.lpush("convertible2", "item").unwrap(), 1);
    assert_eq!(store.llen("convertible2").unwrap(), 1);
    
    // Should error when trying to get as string
    assert!(store.get("convertible2").is_err());
}