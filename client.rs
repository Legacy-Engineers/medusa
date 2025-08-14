use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("⚡ Medusa Client");
    println!("Connecting to server at 127.0.0.1:2312...");

    let mut stream = match TcpStream::connect("127.0.0.1:2312") {
        Ok(stream) => {
            println!("✅ Connected to Medusa server!");
            stream
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to server: {}", e);
            eprintln!("💡 Make sure the Medusa server is running with: cargo run");
            return Err(e);
        }
    };

    // Set socket timeouts (configurable)
    let enable_timeouts = std::env::var("MEDUSA_CLIENT_TIMEOUTS").unwrap_or_else(|_| "false".to_string()) == "true";
    if enable_timeouts {
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    }
    stream.set_nodelay(true)?;

    let read_stream = stream.try_clone()?;
    let (tx, rx) = mpsc::channel();

    // Spawn reader thread
    thread::spawn(move || {
        let mut reader = BufReader::new(read_stream);
        let mut buffer = String::new();

        loop {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("\n🔌 Server disconnected");
                    break;
                }
                Ok(_) => {
                    let response = buffer.trim();
                    if !response.is_empty() {
                        println!("📡 Server: {}", response);
                    }
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("\n❌ Error reading from server: {}", e);
                    break;
                }
            }
        }
        let _ = tx.send(());
    });

    // Main input loop
    let stdin = io::stdin();
    println!("\n🎯 Type commands (or 'help' for available commands, 'quit' to exit):");
    println!("💡 Example: SET user:1 'John Doe' 3600");

    for line in stdin.lock().lines() {
        match line {
            Ok(input) => {
                let trimmed = input.trim();
                
                if trimmed.is_empty() {
                    continue;
                }

                // Handle special commands
                match trimmed.to_lowercase().as_str() {
                    "help" => {
                        print_help();
                        continue;
                    }
                    "quit" | "exit" => {
                        println!("👋 Goodbye!");
                        break;
                    }
                    "clear" => {
                        print!("\x1B[2J\x1B[1;1H"); // Clear screen
                        continue;
                    }
                    _ => {}
                }

                // Send command to server
                let message = format!("{}\n", trimmed);
                if let Err(e) = stream.write_all(message.as_bytes()) {
                    eprintln!("❌ Failed to send message: {}", e);
                    break;
                }

                if trimmed.to_lowercase() == "quit" || trimmed.to_lowercase() == "exit" {
                    break;
                }
            }
            Err(e) => {
                eprintln!("❌ Error reading input: {}", e);
                break;
            }
        }
    }

    // Wait for reader thread to finish
    let _ = rx.recv();
    println!("🔌 Disconnected from server");
    Ok(())
}

fn print_help() {
    println!("\n📚 Available Commands:");
    println!("  SET key value [TTL]     - Store key-value pair with optional TTL");
    println!("  GET key                  - Retrieve value by key");
    println!("  DELETE key               - Remove key-value pair");
    println!("  EXISTS key               - Check if key exists");
    println!("  TTL key                  - Get time-to-live for key");
    println!("  EXPIRE key seconds       - Set expiration time for key");
    println!("  LIST                     - List all keys");
    println!("  KEYS pattern             - Find keys matching pattern");
    println!("  COUNT                    - Get number of entries");
    println!("  CLEAR/FLUSHALL           - Remove all entries");
    println!("  INFO                     - Get server statistics");
    println!("  PING                     - Server health check");
    println!("  QUIT/EXIT                - Disconnect");
    println!("  HELP                     - Show this help");
    println!("  CLEAR                    - Clear screen");
    
    println!("\n🗂️  Hash Operations:");
    println!("  HSET key field value     - Set hash field to value");
    println!("  HGET key field           - Get hash field value");
    println!("  HGETALL key              - Get all hash fields and values");
    println!("  HDEL key field           - Delete hash field");
    println!("  HEXISTS key field        - Check if hash field exists");
    println!("  HLEN key                 - Get hash length");
    
    println!("\n📋 List Operations:");
    println!("  LPUSH key value          - Push value to left of list");
    println!("  RPUSH key value          - Push value to right of list");
    println!("  LPOP key                 - Pop value from left of list");
    println!("  RPOP key                 - Pop value from right of list");
    println!("  LLEN key                 - Get list length");
    println!("  LRANGE key start stop    - Get list range (supports negative indices)");
    
    println!("\n💡 Examples:");
    println!("  SET user:1 'John Doe' 3600    # Set with 1 hour TTL");
    println!("  EXPIRE user:1 7200            # Set 2 hour expiration");
    println!("  KEYS user:*                   # Find all user keys");
    println!("  TTL user:1                    # Check remaining time");
    println!("  HSET user:1 name 'John'       # Set hash field");
    println!("  HGET user:1 name              # Get hash field");
    println!("  LPUSH tasks 'task1'           # Push to list");
    println!("  LRANGE tasks 0 -1             # Get all list items");
    println!();
}
