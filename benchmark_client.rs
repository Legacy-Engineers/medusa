use std::env;
use std::time::Duration;

mod benchmark {
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

    pub fn run_get_benchmark(host: &str, port: u16, operations: usize) -> Result<BenchmarkResult, String> {
        let mut stream = TcpStream::connect(format!("{}:{}", host, port))
            .map_err(|e| format!("Failed to connect: {}", e))?;
        
        // First set some values
        for i in 0..operations {
            let key = format!("get_bench:{}", i);
            let value = format!("get_value_{}", i);
            let command = format!("SET {} {}\n", key, value);
            stream.write_all(command.as_bytes())
                .map_err(|e| format!("Write error: {}", e))?;
            
            let mut buffer = [0; 1024];
            stream.read(&mut buffer)
                .map_err(|e| format!("Read error: {}", e))?;
        }
        
        let start = Instant::now();
        
        // Now benchmark GET operations
        for i in 0..operations {
            let key = format!("get_bench:{}", i);
            let command = format!("GET {}\n", key);
            stream.write_all(command.as_bytes())
                .map_err(|e| format!("Write error: {}", e))?;
            
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
            
            let mut buffer = [0; 1024];
            if let Err(_) = stream.read(&mut buffer) {
                break;
            }
            
            operations += 1;
        }
        
        let duration = start.elapsed();
        Ok(BenchmarkResult::new(operations, duration))
    }
}

fn main() {
    println!("⚡ Medusa Benchmark Client");
    println!("Testing server performance...\n");

    let args: Vec<String> = env::args().collect();
    let host = args.get(1).unwrap_or(&"127.0.0.1".to_string()).clone();
    let port = args.get(2).unwrap_or(&"2312".to_string()).parse::<u16>().unwrap_or(2312);
    let operations = args.get(3).unwrap_or(&"1000".to_string()).parse::<usize>().unwrap_or(1000);
    let threads = args.get(4).unwrap_or(&"4".to_string()).parse::<usize>().unwrap_or(4);

    println!("🎯 Benchmark Configuration:");
    println!("  📍 Host: {}", host);
    println!("  🔌 Port: {}", port);
    println!("  🔢 Operations: {}", operations);
    println!("  🧵 Threads: {}", threads);
    println!();

    // Single-threaded SET benchmark
    println!("🚀 Running single-threaded SET benchmark...");
    match benchmark::run_benchmark(&host, port, operations) {
        Ok(result) => result.display("Single-threaded SET"),
        Err(e) => eprintln!("❌ SET benchmark failed: {}", e),
    }

    // Single-threaded GET benchmark
    println!("🚀 Running single-threaded GET benchmark...");
    match benchmark::run_get_benchmark(&host, port, operations) {
        Ok(result) => result.display("Single-threaded GET"),
        Err(e) => eprintln!("❌ GET benchmark failed: {}", e),
    }

    // Multi-threaded benchmark
    println!("🚀 Running multi-threaded benchmark...");
    match benchmark::run_concurrent_benchmark(&host, port, threads, operations / threads) {
        Ok(results) => {
            let total_ops: usize = results.iter().map(|r| r.operations).sum();
            let total_duration = results.iter().map(|r| r.duration).max().unwrap_or(Duration::ZERO);
            let avg_ops_per_sec: f64 = results.iter().map(|r| r.ops_per_second).sum::<f64>();
            
            println!("📊 Multi-threaded Benchmark Results:");
            println!("  🔢 Total Operations: {}", total_ops);
            println!("  ⏱️  Max Duration: {:?}", total_duration);
            println!("  🚀 Total Ops/sec: {:.2}", avg_ops_per_sec);
            println!();
        }
        Err(e) => eprintln!("❌ Multi-threaded benchmark failed: {}", e),
    }

    // Stress test
    println!("🚀 Running stress test (10 seconds)...");
    match benchmark::run_stress_test(&host, port, 10) {
        Ok(result) => result.display("Stress Test (10s)"),
        Err(e) => eprintln!("❌ Stress test failed: {}", e),
    }

    println!("✅ Benchmark completed!");
} 