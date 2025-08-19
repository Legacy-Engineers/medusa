use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn start_test_server() -> u16 {
    let port = 12312;
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
    thread::sleep(Duration::from_millis(100));
    port
}

fn send_command(port: u16, command: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    
    let mut welcome = String::new();
    stream.read_to_string(&mut welcome)?;
    
    stream.write_all(format!("{}\n", command).as_bytes())?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    
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
    
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
    stream.write_all(b"SET test value\n").unwrap();
    
    drop(stream);
    
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