use std::net::TcpStream;
use std::io::{Write, Read};
use std::time::{Duration, Instant};
use std::thread;

pub struct BenchmarkResult {
    pub operations: usize,
    pub duration: Duration,
    pub ops_per_second: f64,
    pub avg_latency_ms: f64,
}

impl BenchmarkResult {
    pub fn new(operations: usize, duration: Duration) -> Self {
        let ops_per_second = operations as f64 / duration.as_secs_f64();
        let avg_latency_ms = duration.as_millis() as f64 / operations as f64;
        
        BenchmarkResult {
            operations,
            duration,
            ops_per_second,
            avg_latency_ms,
        }
    }
    
    pub fn display(&self, test_name: &str) {
        println!("📊 Benchmark Results: {}", test_name);
        println!("  🔢 Operations: {}", self.operations);
        println!("  ⏱️  Duration: {:?}", self.duration);
        println!("  🚀 Ops/sec: {:.2}", self.ops_per_second);
        println!("  ⚡ Avg Latency: {:.2}ms", self.avg_latency_ms);
        println!();
    }
}

pub fn run_benchmark(host: &str, port: u16, operations: usize) -> Result<BenchmarkResult, String> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let start = Instant::now();
    
    for i in 0..operations {
        let key = format!("bench:{}", i);
        let value = format!("value_{}", i);
        
        let command = format!("SET {} {}\n", key, value);
        stream.write_all(command.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        
        // Read response
        let mut buffer = [0; 1024];
        stream.read(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;
    }
    
    let duration = start.elapsed();
    Ok(BenchmarkResult::new(operations, duration))
}

pub fn run_concurrent_benchmark(host: &str, port: u16, threads: usize, ops_per_thread: usize) -> Result<Vec<BenchmarkResult>, String> {
    let mut handles = vec![];
    let mut results = vec![];
    
    for _ in 0..threads {
        let host = host.to_string();
        let handle = thread::spawn(move || {
            run_benchmark(&host, port, ops_per_thread)
        });
        handles.push(handle);
    }
    
    for handle in handles {
        match handle.join() {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err("Thread join failed".to_string()),
        }
    }
    
    Ok(results)
}

pub fn run_stress_test(host: &str, port: u16, duration_secs: u64) -> Result<BenchmarkResult, String> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let start = Instant::now();
    let target_duration = Duration::from_secs(duration_secs);
    let mut operations = 0;
    
    while start.elapsed() < target_duration {
        let key = format!("stress:{}", operations);
        let value = format!("stress_value_{}", operations);
        
        let command = format!("SET {} {}\n", key, value);
        if let Err(_) = stream.write_all(command.as_bytes()) {
            break;
        }
        
        // Read response
        let mut buffer = [0; 1024];
        if let Err(_) = stream.read(&mut buffer) {
            break;
        }
        
        operations += 1;
    }
    
    let duration = start.elapsed();
    Ok(BenchmarkResult::new(operations, duration))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_result() {
        let result = BenchmarkResult::new(1000, Duration::from_secs(1));
        assert_eq!(result.operations, 1000);
        assert_eq!(result.duration, Duration::from_secs(1));
        assert!((result.ops_per_second - 1000.0).abs() < 0.1);
    }
} 