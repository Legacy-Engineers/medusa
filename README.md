# ⚡ Medusa

<div align="center">

```
███╗   ███╗███████╗██████╗ ██╗   ██╗███████╗ █████╗
████╗ ████║██╔════╝██╔══██╗██║   ██║██╔════╝██╔══██╗
██╔████╔██║█████╗  ██║  ██║██║   ██║███████╗███████║
██║╚██╔╝██║██╔══╝  ██║  ██║██║   ██║╚════██║██╔══██║
██║ ╚═╝ ██║███████╗██████╔╝╚██████╔╝███████║██║  ██║
╚═╝     ╚═╝╚══════╝╚═════╝  ╚═════╝ ╚══════╝╚═╝  ╚═╝
```

_A lightning-fast in-memory key-value store built with Rust_ ⚡

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Experimental-orange?style=for-the-badge)]()

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

## 🛠️ Tech Stack

<div align="center">

|                                       Technology                                       | Purpose                                                 |
| :------------------------------------------------------------------------------------: | :------------------------------------------------------ |
| ![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white) | **Systems Programming** - Memory safety and performance |
|          ![TCP](https://img.shields.io/badge/TCP-Networking-blue?style=flat)           | **Network Layer** - Client-server communication         |
|       ![Memory](https://img.shields.io/badge/Memory-Management-green?style=flat)       | **Data Storage** - In-memory key-value operations       |

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

# Build and run the server
cargo run

# In another terminal, test with the client
rustc client.rs
./client
```

### Example Usage

```bash
# Server will be listening on localhost:6379
# Use the client to interact with Medusa
./client

# Available commands (Redis-like):
SET key value
GET key
DEL key
EXISTS key
```

## 📚 Learning Resources

<div align="center">

|                         Resource                          | Description                                |
| :-------------------------------------------------------: | :----------------------------------------- |
|    [Redis Source Code](https://github.com/redis/redis)    | **The Original** - Study the master        |
| [Build Your Own Redis](https://build-your-own.org/redis/) | **Step-by-step** - C implementation guide  |
|     [The Rust Book](https://doc.rust-lang.org/book/)      | **Language Reference** - Essential reading |

</div>

## 🤝 Contributing

This is a **learning project**, but contributions that improve code quality, documentation, or educational value are welcome!

### Development Guidelines

- 🧪 **Experiment freely** - This is for learning!
- 📝 **Document your discoveries** - Share what you learn
- 🐛 **Report issues** - Help improve the codebase
- 💡 **Suggest improvements** - Better ways to teach concepts

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Built with ⚡ by curious minds for curious minds**

_"The best way to learn is to build"_ 🚀

</div>
