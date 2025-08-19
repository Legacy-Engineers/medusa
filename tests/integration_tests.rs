use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;
use std::time::Duration;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(12312);

fn start_test_server() -> u16 {
    let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
    thread::spawn(move || {
        let config = medusa::server::ServerConfig {
            host: "127.0.0.1".to_string(),
            port,
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            enable_timeouts: false,
        };
        medusa::server::start_server_with_config(config);
    });
    thread::sleep(Duration::from_millis(200)); // Give more time for server to start
    port
}

fn send_command(port: u16, command: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    
    let mut reader = BufReader::new(stream.try_clone()?);
    
    // Read welcome message
    let mut welcome = String::new();
    reader.read_line(&mut welcome)?;
    
    // Send command
    stream.write_all(format!("{}\n", command).as_bytes())?;
    stream.flush()?;
    
    // Read response
    let mut response = String::new();
    reader.read_line(&mut response)?;
    
    Ok(response)
}

#[test]
fn test_basic_operations() {
    let port = start_test_server();
    
    let response = send_command(port, "SET test_key test_value").unwrap();
    assert!(response.contains("OK"));
    
    let response = send_command(port, "GET test_key").unwrap();
    assert!(response.contains("test_value"));
    
    let response = send_command(port, "DELETE test_key").unwrap();
    assert!(response.contains("OK"));
    
    let response = send_command(port, "GET test_key").unwrap();
    assert!(response.contains("NULL"));
}

#[test]
fn test_ttl_operations() {
    let port = start_test_server();
    
    let response = send_command(port, "SET ttl_key ttl_value 1").unwrap();
    assert!(response.contains("OK"));
    
    let response = send_command(port, "TTL ttl_key").unwrap();
    assert!(response.contains("expires in"));
    
    thread::sleep(Duration::from_secs(2));
    
    let response = send_command(port, "GET ttl_key").unwrap();
    assert!(response.contains("NULL"));
}

#[test]
fn test_pattern_matching() {
    let port = start_test_server();
    
    send_command(port, "SET user:1 john").unwrap();
    send_command(port, "SET user:2 jane").unwrap();
    send_command(port, "SET product:1 laptop").unwrap();
    
    let response = send_command(port, "KEYS user:*").unwrap();
    assert!(response.contains("user:1"));
    assert!(response.contains("user:2"));
    assert!(!response.contains("product:1"));
}

#[test]
fn test_connection_resilience() {
    let port = start_test_server();
    
    // First connection - set a value and disconnect abruptly
    {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        
        // Read welcome message
        let mut welcome = String::new();
        reader.read_line(&mut welcome).unwrap();
        
        // Send command
        stream.write_all(b"SET test value\n").unwrap();
        stream.flush().unwrap();
        
        // Read response to ensure command was processed
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();
        
        // Drop the connection
    }
    
    // Second connection - verify the data persisted
    let response = send_command(port, "GET test").unwrap();
    assert!(response.contains("value"));
}

#[test]
fn test_concurrent_connections() {
    let port = start_test_server();
    let mut handles = vec![];
    
    for i in 0..5 {
        let handle = thread::spawn(move || {
            let key = format!("concurrent_key_{}", i);
            let value = format!("value_{}", i);
            
            send_command(port, &format!("SET {} {}", key, value)).unwrap();
            let response = send_command(port, &format!("GET {}", key)).unwrap();
            assert!(response.contains(&value));
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}