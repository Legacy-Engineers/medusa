use crate::store::Store;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;



pub fn handle_client_with_timeout(stream: TcpStream, store: Store, enable_timeouts: bool, timeout: Duration) {
    let client_addr = stream.peer_addr().map(|addr| addr.to_string()).unwrap_or_else(|_| "unknown".to_string());
    println!("New client connected: {}", client_addr);

    if enable_timeouts {
        if let Err(e) = stream.set_read_timeout(Some(timeout)) {
            eprintln!("Failed to set read timeout: {}", e);
            return;
        }
    }

    let read_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to clone stream: {}", e);
            return;
        }
    };
    let mut write_stream = stream;

    let welcome_msg = "Medusa server ready\n";

    let _ = write_stream.write_all(welcome_msg.as_bytes());
    let _ = write_stream.flush();

    let mut reader = BufReader::new(read_stream);
    let mut buffer = String::new();

    loop {
        buffer.clear();

        match reader.read_line(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let message = buffer.trim();
                if message.is_empty() {
                    continue;
                }

                let response = process_command(message, &store);

                if write_stream.write_all(response.as_bytes()).is_err() {
                    break;
                }
                let _ = write_stream.flush();

                if matches!(message.to_lowercase().as_str(), "quit" | "exit") {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn process_command(command: &str, store: &Store) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return "ERROR: Empty command\n".to_string();
    }

    match parts[0].to_uppercase().as_str() {
        "SET" => {
            if parts.len() < 3 {
                return "ERROR: SET requires key and value (SET key value [TTL seconds])\n".to_string();
            }
            let key = parts[1];
            let value = parts[2..].join(" ");

            // Check if TTL is provided
            if let Some(ttl_part) = parts.last() {
                if let Ok(ttl_seconds) = ttl_part.parse::<u64>() {
                    // Remove TTL from value if it was parsed as TTL
                    let value_without_ttl = parts[2..parts.len()-1].join(" ");
                    match store.set_with_ttl(key, &value_without_ttl, ttl_seconds) {
                        Ok(_) => format!("OK: Set '{}' = '{}' with TTL {}s\n", key, value_without_ttl, ttl_seconds),
                        Err(e) => format!("ERROR: Failed to set value: {}\n", e),
                    }
                } else {
                    match store.set(key, &value) {
                        Ok(_) => format!("OK: Set '{}' = '{}'\n", key, value),
                        Err(e) => format!("ERROR: Failed to set value: {}\n", e),
                    }
                }
            } else {
                // No TTL provided
                match store.set(key, &value) {
                    Ok(_) => format!("OK: Set '{}' = '{}'\n", key, value),
                    Err(e) => format!("ERROR: Failed to set value: {}\n", e),
                }
            }
        }

        "GET" => {
            if parts.len() < 2 {
                return "ERROR: GET requires a key (GET key)\n".to_string();
            }
            let key = parts[1];

            match store.get(key) {
                Ok(Some(value)) => format!("OK: '{}' = {}\n", key, value),
                Ok(None) => format!("NULL: Key '{}' not found or expired\n", key),
                Err(e) => format!("ERROR: Failed to get value: {}\n", e),
            }
        }

        "DELETE" => {
            if parts.len() < 2 {
                return "ERROR: DELETE requires a key (DELETE key)\n".to_string();
            }
            let key = parts[1];

            match store.delete(key) {
                Ok(Some(value)) => format!("OK: Deleted '{}' (was '{}')\n", key, value),
                Ok(None) => format!("NULL: Key '{}' not found\n", key),
                Err(e) => format!("ERROR: Failed to delete: {}\n", e),
            }
        }

        "EXISTS" => {
            if parts.len() < 2 {
                return "ERROR: EXISTS requires a key (EXISTS key)\n".to_string();
            }
            let key = parts[1];

            match store.exists(key) {
                Ok(true) => format!("TRUE: Key '{}' exists\n", key),
                Ok(false) => format!("FALSE: Key '{}' does not exist\n", key),
                Err(e) => format!("ERROR: Failed to check existence: {}\n", e),
            }
        }

        "TTL" => {
            if parts.len() < 2 {
                return "ERROR: TTL requires a key (TTL key)\n".to_string();
            }
            let key = parts[1];

            match store.ttl(key) {
                Ok(Some(ttl)) => {
                    if ttl == -1 {
                        format!("TTL: Key '{}' has expired\n", key)
                    } else {
                        format!("TTL: Key '{}' expires in {} seconds\n", key, ttl)
                    }
                }
                Ok(None) => format!("NULL: Key '{}' not found\n", key),
                Err(e) => format!("ERROR: Failed to get TTL: {}\n", e),
            }
        }

        "EXPIRE" => {
            if parts.len() < 3 {
                return "ERROR: EXPIRE requires key and seconds (EXPIRE key seconds)\n".to_string();
            }
            let key = parts[1];
            let ttl_seconds = match parts[2].parse::<u64>() {
                Ok(seconds) => seconds,
                Err(_) => return "ERROR: Invalid TTL value\n".to_string(),
            };

            match store.expire(key, ttl_seconds) {
                Ok(true) => format!("OK: Set expiration for '{}' to {} seconds\n", key, ttl_seconds),
                Ok(false) => format!("FALSE: Key '{}' not found\n", key),
                Err(e) => format!("ERROR: Failed to set expiration: {}\n", e),
            }
        }

        "LIST" => match store.list_keys() {
            Ok(keys) => {
                if keys.is_empty() {
                    "OK: No keys found\n".to_string()
                } else {
                    format!("OK: Keys: {}\n", keys.join(", "))
                }
            }
            Err(e) => format!("ERROR: Failed to list keys: {}\n", e),
        },

        "KEYS" => {
            if parts.len() < 2 {
                return "ERROR: KEYS requires a pattern (KEYS pattern)\n".to_string();
            }
            let pattern = parts[1];

            match store.keys(pattern) {
                Ok(keys) => {
                    if keys.is_empty() {
                        format!("OK: No keys matching pattern '{}'\n", pattern)
                    } else {
                        format!("OK: Keys matching '{}': {}\n", pattern, keys.join(", "))
                    }
                }
                Err(e) => format!("ERROR: Failed to find keys: {}\n", e),
            }
        }

        "COUNT" => match store.count() {
            Ok(count) => format!("OK: {} entries\n", count),
            Err(e) => format!("ERROR: Failed to count entries: {}\n", e),
        },

        "CLEAR" | "FLUSHALL" => match store.clear() {
            Ok(_) => "OK: All entries cleared\n".to_string(),
            Err(e) => format!("ERROR: Failed to clear: {}\n", e),
        },

        "INFO" => match store.info() {
            Ok(info) => format!("OK: Server Info:\n{}\n", info),
            Err(e) => format!("ERROR: Failed to get info: {}\n", e),
        },

        "PING" => "PONG\n".to_string(),

        "QUIT" | "EXIT" => "OK: Goodbye!\n".to_string(),

        // Hash operations
        "HSET" => {
            if parts.len() < 4 {
                return "ERROR: HSET requires key, field, and value (HSET key field value)\n".to_string();
            }
            let key = parts[1];
            let field = parts[2];
            let value = parts[3..].join(" ");

            match store.hset(key, field, &value) {
                Ok(is_new) => {
                    if is_new {
                        format!("OK: Created new field '{}' in hash '{}'\n", field, key)
                    } else {
                        format!("OK: Updated field '{}' in hash '{}'\n", field, key)
                    }
                }
                Err(e) => format!("ERROR: Failed to set hash field: {}\n", e),
            }
        }

        "HGET" => {
            if parts.len() < 3 {
                return "ERROR: HGET requires key and field (HGET key field)\n".to_string();
            }
            let key = parts[1];
            let field = parts[2];

            match store.hget(key, field) {
                Ok(Some(value)) => format!("OK: '{}:{}' = {}\n", key, field, value),
                Ok(None) => format!("NULL: Field '{}' not found in hash '{}'\n", field, key),
                Err(e) => format!("ERROR: Failed to get hash field: {}\n", e),
            }
        }

        "HGETALL" => {
            if parts.len() < 2 {
                return "ERROR: HGETALL requires a key (HGETALL key)\n".to_string();
            }
            let key = parts[1];

            match store.hgetall(key) {
                Ok(fields) => {
                    if fields.is_empty() {
                        format!("OK: Hash '{}' is empty\n", key)
                    } else {
                        let field_list: Vec<String> = fields.iter()
                            .map(|(k, v)| format!("{}:{}", k, v))
                            .collect();
                        format!("OK: Hash '{}' fields: {}\n", key, field_list.join(", "))
                    }
                }
                Err(e) => format!("ERROR: Failed to get hash: {}\n", e),
            }
        }

        "HDEL" => {
            if parts.len() < 3 {
                return "ERROR: HDEL requires key and field (HDEL key field)\n".to_string();
            }
            let key = parts[1];
            let field = parts[2];

            match store.hdel(key, field) {
                Ok(true) => format!("OK: Deleted field '{}' from hash '{}'\n", field, key),
                Ok(false) => format!("FALSE: Field '{}' not found in hash '{}'\n", field, key),
                Err(e) => format!("ERROR: Failed to delete hash field: {}\n", e),
            }
        }

        "HEXISTS" => {
            if parts.len() < 3 {
                return "ERROR: HEXISTS requires key and field (HEXISTS key field)\n".to_string();
            }
            let key = parts[1];
            let field = parts[2];

            match store.hexists(key, field) {
                Ok(true) => format!("TRUE: Field '{}' exists in hash '{}'\n", field, key),
                Ok(false) => format!("FALSE: Field '{}' does not exist in hash '{}'\n", field, key),
                Err(e) => format!("ERROR: Failed to check hash field existence: {}\n", e),
            }
        }

        "HLEN" => {
            if parts.len() < 2 {
                return "ERROR: HLEN requires a key (HLEN key)\n".to_string();
            }
            let key = parts[1];

            match store.hlen(key) {
                Ok(len) => format!("OK: Hash '{}' has {} fields\n", key, len),
                Err(e) => format!("ERROR: Failed to get hash length: {}\n", e),
            }
        }

        // List operations
        "LPUSH" => {
            if parts.len() < 3 {
                return "ERROR: LPUSH requires key and value (LPUSH key value)\n".to_string();
            }
            let key = parts[1];
            let value = parts[2..].join(" ");

            match store.lpush(key, &value) {
                Ok(len) => format!("OK: Pushed to left of list '{}', new length: {}\n", key, len),
                Err(e) => format!("ERROR: Failed to push to list: {}\n", e),
            }
        }

        "RPUSH" => {
            if parts.len() < 3 {
                return "ERROR: RPUSH requires key and value (RPUSH key value)\n".to_string();
            }
            let key = parts[1];
            let value = parts[2..].join(" ");

            match store.rpush(key, &value) {
                Ok(len) => format!("OK: Pushed to right of list '{}', new length: {}\n", key, len),
                Err(e) => format!("ERROR: Failed to push to list: {}\n", e),
            }
        }

        "LPOP" => {
            if parts.len() < 2 {
                return "ERROR: LPOP requires a key (LPOP key)\n".to_string();
            }
            let key = parts[1];

            match store.lpop(key) {
                Ok(Some(value)) => format!("OK: Popped from left of list '{}': {}\n", key, value),
                Ok(None) => format!("NULL: List '{}' is empty\n", key),
                Err(e) => format!("ERROR: Failed to pop from list: {}\n", e),
            }
        }

        "RPOP" => {
            if parts.len() < 2 {
                return "ERROR: RPOP requires a key (RPOP key)\n".to_string();
            }
            let key = parts[1];

            match store.rpop(key) {
                Ok(Some(value)) => format!("OK: Popped from right of list '{}': {}\n", key, value),
                Ok(None) => format!("NULL: List '{}' is empty\n", key),
                Err(e) => format!("ERROR: Failed to pop from list: {}\n", e),
            }
        }

        "LLEN" => {
            if parts.len() < 2 {
                return "ERROR: LLEN requires a key (LLEN key)\n".to_string();
            }
            let key = parts[1];

            match store.llen(key) {
                Ok(len) => format!("OK: List '{}' has {} items\n", key, len),
                Err(e) => format!("ERROR: Failed to get list length: {}\n", e),
            }
        }

        "LRANGE" => {
            if parts.len() < 4 {
                return "ERROR: LRANGE requires key, start, and stop (LRANGE key start stop)\n".to_string();
            }
            let key = parts[1];
            let start = match parts[2].parse::<i64>() {
                Ok(s) => s,
                Err(_) => return "ERROR: Invalid start index\n".to_string(),
            };
            let stop = match parts[3].parse::<i64>() {
                Ok(s) => s,
                Err(_) => return "ERROR: Invalid stop index\n".to_string(),
            };

            match store.lrange(key, start, stop) {
                Ok(items) => {
                    if items.is_empty() {
                        format!("OK: No items in range [{}, {}] for list '{}'\n", start, stop, key)
                    } else {
                        format!("OK: List '{}' range [{}, {}]: {}\n", key, start, stop, items.join(", "))
                    }
                }
                Err(e) => format!("ERROR: Failed to get list range: {}\n", e),
            }
        }

        _ => {
            format!("ERROR: Unknown command '{}'\n", parts[0])
        }
    }
}
