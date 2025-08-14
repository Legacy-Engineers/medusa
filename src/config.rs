use std::env;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub enable_timeouts: bool,
    pub log_level: String,
    pub enable_metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            port: 2312,
            max_connections: 100,
            connection_timeout: Duration::from_secs(30),
            enable_timeouts: false,
            log_level: "info".to_string(),
            enable_metrics: false,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut config = Config::default();
        
        // Load from environment variables
        if let Ok(host) = env::var("MEDUSA_HOST") {
            config.host = host;
        }
        
        if let Ok(port) = env::var("MEDUSA_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.port = port_num;
            }
        }
        
        if let Ok(max_conn) = env::var("MEDUSA_MAX_CONNECTIONS") {
            if let Ok(max_conn_num) = max_conn.parse::<usize>() {
                config.max_connections = max_conn_num;
            }
        }
        
        if let Ok(timeout) = env::var("MEDUSA_TIMEOUT") {
            if let Ok(timeout_secs) = timeout.parse::<u64>() {
                config.connection_timeout = Duration::from_secs(timeout_secs);
            }
        }
        
        if let Ok(enable_timeouts) = env::var("MEDUSA_ENABLE_TIMEOUTS") {
            config.enable_timeouts = enable_timeouts.to_lowercase() == "true";
        }
        
        if let Ok(log_level) = env::var("MEDUSA_LOG_LEVEL") {
            config.log_level = log_level;
        }
        
        if let Ok(metrics) = env::var("MEDUSA_METRICS") {
            config.enable_metrics = metrics.to_lowercase() == "true";
        }
        
        config
    }
    
    pub fn display(&self) {
        println!("⚙️  Medusa Configuration:");
        println!("  📍 Host: {}", self.host);
        println!("  🔌 Port: {}", self.port);
        println!("  🔗 Max Connections: {}", self.max_connections);
        println!("  ⏱️  Timeouts: {}", if self.enable_timeouts { "Enabled" } else { "Disabled" });
        if self.enable_timeouts {
            println!("  ⏱️  Timeout Duration: {:?}", self.connection_timeout);
        }
        println!("  📊 Log Level: {}", self.log_level);
        println!("  📈 Metrics: {}", self.enable_metrics);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 2312);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.connection_timeout, Duration::from_secs(30));
        assert_eq!(config.enable_timeouts, false);
    }
    
    #[test]
    fn test_config_from_env() {
        // Test that it doesn't panic even without env vars
        let config = Config::from_env();
        assert!(!config.host.is_empty());
        assert!(config.port > 0);
    }
} 