# ⚡ Medusa

<div align="center">

```
███╗   ███╗███████╗██████╗ ██╗   ██╗███████╗ █████╗
████╗ ████║██╔════╝██╔══██╗██║   ██║██╔════╝██╔══██╗
██╔████╔██║█████╗  ██║  ██║██║   ██║███████╗███████║
██║╚██╔╝██║██╔══╝  ██║  ██║██║   ██║╚════██║██╔══██║
██║ ╚═╝ ██║███████╗██████╔╝╚██████╔╝███████║██║  ██║
╚═╝     ╚═╝╚══════╝╚═════╝  ╚═════╝ ⚠══════╝╚═╝  ╚═╝
```

_A lightning-fast in-memory key-value store built with Rust_ ⚡

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Experimental-orange?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/Version-0.1.0-blue?style=for-the-badge)]()

</div>

---

## 🎯 About

**Medusa** is a lightweight, high-performance in-memory key-value store inspired by [Redis](https://redis.io), built entirely in **Rust**. This project serves as a learning experiment to explore systems programming concepts and database internals.

### 🧠 What You'll Learn

- **Low-level data storage** and memory management
- **TCP networking** and concurrent client handling
- **Command parsing** and response serialization
- **Systems programming** with Rust's safety guarantees
- **Database internals** and performance optimization
- **TTL (Time To Live)** and expiration handling
- **Pattern matching** and advanced queries
- **Benchmarking** and performance testing

## ⚡ Why "Medusa"?

Just as the mythical Medusa possessed incredible power, this project channels that energy into a compact, efficient data store. While it won't turn you to stone, it might transform your understanding of how databases work under the hood! 🧠⚙️

## 🎯 Project Goals

<div align="center">

|         🎓 **Learning**         |    ⚡ **Performance**     |    🛠️ **Hands-on**    |
| :-----------------------------: | :-----------------------: | :-------------------: |
|   Explore database internals    | Fast in-memory operations |  Build from scratch   |
| Master Rust systems programming |  Efficient TCP handling   | Understand every line |
| Study Redis-like architectures  | Minimal memory footprint  |    Learn by doing     |

</div>

## 🚫 What This Is NOT

> ⚠️ **Important**: This is **not** a production-ready Redis clone.

**Missing Features:**

- ❌ Clustering and replication
- ❌ Persistence and snapshots
- ❌ Advanced data types (sets, sorted sets, etc.)
- ❌ Pub/sub messaging
- ❌ Lua scripting

**Focus:** Understanding core concepts, not feature completeness.

## 🆕 New Features

### ⏱️ **TTL Support**

- Set expiration times for keys
- Automatic cleanup of expired keys
- TTL querying and management

### 🔍 **Pattern Matching**

- Wildcard key searches (`KEYS user:*`)
- Advanced pattern matching capabilities
- Efficient key filtering

### ⚙️ **Configuration System**

- Environment variable support
- Configurable timeouts and limits
- Flexible server settings

### 📊 **Benchmarking Tools**

- Built-in performance testing
- Multi-threaded benchmarks
- Stress testing capabilities

### 🛡️ **Enhanced Error Handling**

- Better connection management
- Timeout support
- Graceful error recovery

## 🛠️ Tech Stack

<div align="center">

|                                       Technology                                       | Purpose                                                 |
| :------------------------------------------------------------------------------------: | :------------------------------------------------------ |
| ![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white) | **Systems Programming** - Memory safety and performance |
|          ![TCP](https://img.shields.io/badge/TCP-Networking-blue?style=flat)           | **Network Layer** - Client-server communication         |
|       ![Memory](https://img.shields.io/badge/Memory-Management-green?style=flat)       | **Data Storage** - In-memory key-value operations       |
|         ![TTL](https://img.shields.io/badge/TTL-Expiration-orange?style=flat)          | **Time Management** - Automatic key expiration          |

</div>

## 🚀 Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- A terminal and some curiosity! 🧠

### Running Medusa

```bash
# Clone and navigate to the project
git clone <your-repo-url>
cd medusa

# Build the project
cargo build --release

# Run the server
cargo run --release

# Or run the optimized binary
./target/release/medusa
```

### Using the Client

```bash
# Run the interactive client
cargo run --bin medusa-client

# Or use the compiled binary
./target/release/medusa-client
```

### Benchmarking

```bash
# Run performance benchmarks
cargo run --bin medusa-benchmark

# Custom benchmark parameters
cargo run --bin medusa-benchmark 127.0.0.1 2312 10000 8
```

## 📚 Available Commands

### 🔧 **Basic Operations**

```bash
SET key value [TTL seconds]  # Store key-value pair with optional TTL
GET key                      # Retrieve value by key
DELETE key                   # Remove key-value pair
EXISTS key                   # Check if key exists
```

### ⏱️ **TTL Management**

```bash
TTL key                      # Get time-to-live for key
EXPIRE key seconds           # Set expiration time for key
```

### 🔍 **Query Operations**

```bash
LIST                         # List all keys
KEYS pattern                 # Find keys matching pattern (use * for wildcard)
COUNT                        # Get number of entries
```

### 🛠️ **Administrative**

```bash
CLEAR/FLUSHALL              # Remove all entries
INFO                         # Get server statistics
PING                         # Server health check
QUIT/EXIT                    # Disconnect
```

### 💡 **Examples**

```bash
SET user:1 "John Doe" 3600    # Set with 1 hour TTL
EXPIRE user:1 7200            # Set 2 hour expiration
KEYS user:*                   # Find all user keys
TTL user:1                    # Check remaining time
```

## ⚙️ Configuration

### Environment Variables

```bash
export MEDUSA_HOST="127.0.0.1"
export MEDUSA_PORT="2312"
export MEDUSA_MAX_CONNECTIONS="100"
export MEDUSA_TIMEOUT="30"
export MEDUSA_LOG_LEVEL="info"
export MEDUSA_METRICS="false"
```

### Server Configuration

The server supports various configuration options:

- **Host/Port**: Customize server binding
- **Max Connections**: Limit concurrent clients
- **Timeouts**: Configure connection timeouts
- **Logging**: Adjust verbosity levels

## 📊 Performance

### Benchmark Results

Typical performance on modern hardware:

```
📊 Benchmark Results: Single-threaded SET
  🔢 Operations: 10000
  ⏱️  Duration: 1.2s
  🚀 Ops/sec: 8333.33
  ⚡ Avg Latency: 0.12ms

📊 Benchmark Results: Multi-threaded (4 threads)
  🔢 Total Operations: 40000
  🚀 Total Ops/sec: 25000.00
```

### Performance Features

- **Lock-free operations** where possible
- **Efficient memory management**
- **Optimized TCP handling**
- **Automatic expired key cleanup**

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test store
cargo test server

# Run benchmarks
cargo run --bin medusa-benchmark
```

## 📚 Learning Resources

<div align="center">

|                          Resource                          | Description                                   |
| :--------------------------------------------------------: | :-------------------------------------------- |
|    [Redis Source Code](https://github.com/redis/redis)     | **The Original** - Study the master           |
| [Build Your Own Redis](https://build-your-own.org/redis/)  | **Step-by-step** - C implementation guide     |
|      [The Rust Book](https://doc.rust-lang.org/book/)      | **Language Reference** - Essential reading    |
| [Rust Async Book](https://rust-lang.github.io/async-book/) | **Async Programming** - For advanced features |

</div>

## 🤝 Contributing

This is a **learning project**, but contributions that improve code quality, documentation, or educational value are welcome!

### Development Guidelines

- 🧪 **Experiment freely** - This is for learning!
- 📝 **Document your discoveries** - Share what you learn
- 🐛 **Report issues** - Help improve the codebase
- 💡 **Suggest improvements** - Better ways to teach concepts

### Code Style

- Follow Rust conventions
- Add tests for new features
- Update documentation
- Use meaningful commit messages

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Built with ⚡ by curious minds for curious minds**

_"The best way to learn is to build"_ 🚀

</div>
