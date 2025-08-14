#!/bin/bash

echo "🚀 Medusa New Features Demo"
echo "=========================="
echo ""

# Start the server in the background
echo "Starting Medusa server..."
cargo run --release &
SERVER_PID=$!

# Wait for server to start
sleep 2

echo "Server started with PID: $SERVER_PID"
echo ""

# Test Hash Operations
echo "🗂️  Testing Hash Operations:"
echo "----------------------------"
echo "HSET user:1 name 'John Doe'"
echo "HSET user:1 name 'John Doe'" | nc 127.0.0.1 2312

echo "HSET user:1 age '30'"
echo "HSET user:1 age '30'" | nc 127.0.0.1 2312

echo "HSET user:1 email 'john@example.com'"
echo "HSET user:1 email 'john@example.com'" | nc 127.0.0.1 2312

echo "HGET user:1 name"
echo "HGET user:1 name" | nc 127.0.0.1 2312

echo "HGETALL user:1"
echo "HGETALL user:1" | nc 127.0.0.1 2312

echo "HLEN user:1"
echo "HLEN user:1" | nc 127.0.0.1 2312

echo "HEXISTS user:1 age"
echo "HEXISTS user:1 age" | nc 127.0.0.1 2312

echo "HDEL user:1 age"
echo "HDEL user:1 age" | nc 127.0.0.1 2312

echo "HGETALL user:1 (after deletion)"
echo "HGETALL user:1" | nc 127.0.0.1 2312

echo ""

# Test List Operations
echo "📋 Testing List Operations:"
echo "---------------------------"
echo "LPUSH tasks 'Complete project'"
echo "LPUSH tasks 'Complete project'" | nc 127.0.0.1 2312

echo "LPUSH tasks 'Review code'"
echo "LPUSH tasks 'Review code'" | nc 127.0.0.1 2312

echo "RPUSH tasks 'Write tests'"
echo "RPUSH tasks 'Write tests'" | nc 127.0.0.1 2312

echo "RPUSH tasks 'Deploy'"
echo "RPUSH tasks 'Deploy'" | nc 127.0.0.1 2312

echo "LLEN tasks"
echo "LLEN tasks" | nc 127.0.0.1 2312

echo "LRANGE tasks 0 -1"
echo "LRANGE tasks 0 -1" | nc 127.0.0.1 2312

echo "LPOP tasks"
echo "LPOP tasks" | nc 127.0.0.1 2312

echo "RPOP tasks"
echo "RPOP tasks" | nc 127.0.0.1 2312

echo "LRANGE tasks 0 -1 (after pops)"
echo "LRANGE tasks 0 -1" | nc 127.0.0.1 2312

echo ""

# Test Type Conflicts
echo "⚠️  Testing Type Conflicts:"
echo "---------------------------"
echo "SET string_key 'hello'"
echo "SET string_key 'hello'" | nc 127.0.0.1 2312

echo "HSET string_key field 'value' (should fail)"
echo "HSET string_key field 'value'" | nc 127.0.0.1 2312

echo "LPUSH string_key 'item' (should fail)"
echo "LPUSH string_key 'item'" | nc 127.0.0.1 2312

echo ""

# Show all keys
echo "🔑 All Keys:"
echo "-----------"
echo "KEYS *"
echo "KEYS *" | nc 127.0.0.1 2312

echo ""

# Cleanup
echo "🧹 Cleaning up..."
echo "CLEAR"
echo "CLEAR" | nc 127.0.0.1 2312

echo "QUIT"
echo "QUIT" | nc 127.0.0.1 2312

# Stop the server
echo "Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo ""
echo "✅ Demo completed!"
