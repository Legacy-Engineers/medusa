use medusa::config::Config;
use medusa::server::{start_server_with_config, ServerConfig};

fn main() {
    println!("[:)] Medusa - Lightning Fast Key-Value Store");
    println!("Built with Rust for learning and experimentation\n");

    let config = Config::from_env();
    config.display();

    let server_config = ServerConfig {
        host: config.host,
        port: config.port,
        max_connections: config.max_connections,
        connection_timeout: config.connection_timeout,
        enable_timeouts: config.enable_timeouts,
    };

    // Start the server
    start_server_with_config(server_config);
}
