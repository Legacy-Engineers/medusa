use crate::client_handler::handle_client_with_timeout;
use crate::store::Store;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub enable_timeouts: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 2312,
            max_connections: 100,
            connection_timeout: Duration::from_secs(30),
            enable_timeouts: false,
        }
    }
}

pub fn start_server_with_config(config: ServerConfig) {
    let address = format!("{}:{}", config.host, config.port);
    
    println!("⚡ Starting Medusa server...");
    println!("📍 Address: {}", address);
    println!("🔗 Max connections: {}", config.max_connections);
    println!("⏱️  Timeouts: {}", if config.enable_timeouts { "Enabled" } else { "Disabled" });
    if config.enable_timeouts {
        println!("⏱️  Connection timeout: {:?}", config.connection_timeout);
    }
    
    let listener = match TcpListener::bind(&address) {
        Ok(listener) => {
            println!("✅ Server bound successfully to {}", address);
            listener
        }
        Err(e) => {
            eprintln!("❌ Failed to bind to {}: {}", address, e);
            return;
        }
    };

    // Set socket options for better performance
    if let Err(e) = listener.set_nonblocking(false) {
        eprintln!("⚠️  Warning: Could not set non-blocking mode: {}", e);
    }

    let store = Store::new();
    let mut connection_count = 0;

    println!("🚀 Medusa server is ready! Waiting for connections...\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                connection_count += 1;
                
                if connection_count > config.max_connections {
                    eprintln!("⚠️  Max connections reached ({}), rejecting new connection", config.max_connections);
                    continue;
                }

                // Set socket options for the client connection
                if config.enable_timeouts {
                    if let Err(e) = configure_client_socket(&stream, config.connection_timeout) {
                        eprintln!("⚠️  Warning: Could not configure client socket: {}", e);
                    }
                }

                let store_clone = store.clone();
                let client_addr = match stream.peer_addr() {
                    Ok(addr) => addr.to_string(),
                    Err(_) => "unknown".to_string(),
                };

                println!("🔌 New connection #{} from {}", connection_count, client_addr);

                thread::spawn(move || {
                    handle_client_with_timeout(stream, store_clone, config.enable_timeouts, config.connection_timeout);
                    println!("🔌 Connection #{} from {} closed", connection_count, client_addr);
                });
            }
            Err(e) => {
                eprintln!("❌ Failed to accept connection: {}", e);
            }
        }
    }
}

fn configure_client_socket(stream: &TcpStream, timeout: Duration) -> std::io::Result<()> {
    // Set read timeout
    stream.set_read_timeout(Some(timeout))?;
    
    // Set write timeout
    stream.set_write_timeout(Some(timeout))?;
    
    // Set TCP nodelay for better performance
    stream.set_nodelay(true)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::time::Duration;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 2312);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.connection_timeout, Duration::from_secs(30));
        assert_eq!(config.enable_timeouts, false);
    }

    #[test]
    fn test_socket_configuration() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        
        // Test client connection
        let client_stream = TcpStream::connect(addr).unwrap();
        let result = configure_client_socket(&client_stream, Duration::from_secs(10));
        assert!(result.is_ok());
    }
}
