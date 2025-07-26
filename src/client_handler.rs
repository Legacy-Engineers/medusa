use crate::store::Store;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

pub fn handle_client(stream: TcpStream, store: Store) {
    println!("New client connected: {}", stream.peer_addr().unwrap());

    // Set socket timeout for better connection handling
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(30))) {
        eprintln!("Failed to set read timeout: {}", e);
    }

    let read_stream = stream.try_clone().expect("Failed to clone stream");
    let mut write_stream = stream;

    let welcome_msg = r#"⚡ Welcome to Medusa server!
Commands:
  SET key value [TTL seconds]  - Store a key-value pair with optional TTL
  GET key                      - Retrieve value by key
  DELETE key                   - Remove key-value pair
  EXISTS key                   - Check if key exists
  TTL key                      - Get time-to-live for key
  EXPIRE key seconds           - Set expiration time for key
  LIST                         - List all keys
  KEYS pattern                 - Find keys matching pattern (use * for wildcard)
  COUNT                        - Get number of entries
  CLEAR                        - Remove all entries
  FLUSHALL                     - Remove all entries (alias for CLEAR)
  INFO                         - Get server statistics
  PING                         - Server health check
  QUIT/EXIT                    - Disconnect

Examples:
  SET user:1 "John Doe" 3600    # Set with 1 hour TTL
  EXPIRE user:1 7200            # Set 2 hour expiration
  KEYS user:*                   # Find all user keys
  TTL user:1                    # Check remaining time

"#;

    if let Err(e) = write_stream.write_all(welcome_msg.as_bytes()) {
        eprintln!("Failed to send welcome message: {}", e);
        return;
    }

    let mut reader = BufReader::new(read_stream);
    let mut buffer = String::new();

    loop {
        buffer.clear();

        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(_) => {
                let message = buffer.trim();
                println!("Received: {}", message);

                let response = process_command(message, &store);

                if let Err(e) = write_stream.write_all(response.as_bytes()) {
                    eprintln!("Failed to send response: {}", e);
                    break;
                }

                if message.to_lowercase() == "quit" || message.to_lowercase() == "exit" {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        }
    }

    println!("Client handler finished");
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
                    // No TTL provided, set normally
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

        _ => {
            format!("ERROR: Unknown command '{}'\n", parts[0])
        }
    }
}
