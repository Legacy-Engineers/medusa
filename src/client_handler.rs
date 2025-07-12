use crate::store::Store;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub fn handle_client(stream: TcpStream, store: Store) {
    println!("New client connected: {}", stream.peer_addr().unwrap());

    let read_stream = stream.try_clone().expect("Failed to clone stream");
    let mut write_stream = stream;

    let welcome_msg = r#"Welcome to Medusa server!
Commands:
  SET key value    - Store a key-value pair
  GET key          - Retrieve value by key
  DELETE key       - Remove key-value pair
  EXISTS key       - Check if key exists
  LIST             - List all keys
  COUNT            - Get number of entries
  CLEAR            - Remove all entries
  PING             - Server health check
  QUIT/EXIT        - Disconnect

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
                return "ERROR: SET requires key and value (SET key value)\n".to_string();
            }
            let key = parts[1];
            let value = parts[2..].join(" ");

            match store.set(key, &value) {
                Ok(_) => format!("OK: Set '{}' = '{}'\n", key, value),
                Err(e) => format!("ERROR: Failed to set value: {}\n", e),
            }
        }

        "GET" => {
            if parts.len() < 2 {
                return "ERROR: GET requires a key (GET key)\n".to_string();
            }
            let key = parts[1];

            match store.get(key) {
                Ok(Some(value)) => format!("OK: '{}' = {} \n", key, value),
                Ok(None) => format!("NULL: Key '{}' not found\n", key),
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

        "COUNT" => match store.count() {
            Ok(count) => format!("OK: {} entries\n", count),
            Err(e) => format!("ERROR: Failed to count entries: {}\n", e),
        },

        "CLEAR" => match store.clear() {
            Ok(_) => "OK: All entries cleared\n".to_string(),
            Err(e) => format!("ERROR: Failed to clear: {}\n", e),
        },

        "PING" => "PONG\n".to_string(),

        "QUIT" | "EXIT" => "OK: Goodbye!\n".to_string(),

        _ => {
            format!("ERROR: Unknown command '{}'\n", parts[0])
        }
    }
}
