# Redis Storage Example

This example demonstrates how to use Redis as a storage backend with PocketFlow-rs.

## Prerequisites

You need a Redis server running to use this example. You have several options:

### Option 1: Docker (Recommended)

```bash
# Start a Redis server with Docker
docker run --rm -p 6379:6379 redis:latest
```

### Option 2: Local Redis Installation

#### macOS (with Homebrew)
```bash
brew install redis
brew services start redis
```

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install redis-server
sudo systemctl start redis-server
```

#### Redis Cloud
You can also use a cloud Redis service and modify the connection URL in the example.

## Running the Example

1. Make sure Redis is running (see prerequisites above)

2. Run the example with Redis feature enabled:
```bash
cargo run --example redis_storage --features redis
```

## What the Example Demonstrates

1. **Redis Connection**: How to connect to Redis using different configurations
2. **Basic Operations**: Set, get, remove, and check key existence
3. **Complex Data Storage**: Storing JSON objects and nested data structures
4. **Workflow State Management**: Using Redis for persistent workflow state
5. **Flow Execution**: Running complete PocketFlow workflows with Redis storage
6. **Key Management**: Listing keys, counting items, and cleanup operations

## Redis Storage Features

- **Persistence**: Data survives application restarts
- **Scalability**: Can be used in distributed environments
- **Key Prefixing**: Automatic namespace management with configurable prefixes
- **JSON Serialization**: Automatic conversion between Rust types and JSON
- **Error Handling**: Comprehensive error handling for connection and operation failures

## Configuration Options

```rust
// Basic connection
let storage = RedisStorage::new("redis://localhost:6379/")?;

// With custom prefix
let storage = RedisStorage::new_with_prefix("redis://localhost:6379/", "myapp")?;

// With authentication (if needed)
let storage = RedisStorage::new("redis://:password@localhost:6379/")?;

// Remote Redis (e.g., Redis Cloud)
let storage = RedisStorage::new("redis://user:pass@remote-host:6379/")?;
```

## Troubleshooting

### Connection Refused
If you get a connection refused error:
- Make sure Redis is running: `redis-cli ping` (should return "PONG")
- Check if Redis is listening on the correct port: `netstat -an | grep 6379`
- Verify firewall settings if using remote Redis

### Permission Denied
If you get permission errors:
- Check Redis configuration file (`/etc/redis/redis.conf`)
- Ensure the user has appropriate permissions
- For cloud Redis, verify authentication credentials

### Performance Considerations
- Redis operations are network-bound, so minimize round trips
- Use batch operations when possible
- Consider Redis connection pooling for high-throughput applications
- Monitor Redis memory usage and configure appropriate eviction policies

## Example Output

When you run the example successfully, you should see output similar to:

```
🚀 PocketFlow-RS Redis Storage Example
🔗 Connecting to Redis at: redis://127.0.0.1:6379/
✅ Connected to Redis successfully!

📝 Example 1: Basic Redis Storage Operations
🧹 Cleared existing data
✅ Stored user session data in Redis
👤 Retrieved user session: {"login_time":"2024-01-15T10:30:00Z",...}

🤖 Example 2: LLM Configuration Storage
✅ Stored 3 LLM configurations
🔑 All keys in Redis: ["user_session", "llm_config_gpt4_creative", ...]

⚡ Example 3: Workflow State Management
✅ Stored workflow state data

🔄 Example 4: Running a Flow with Redis Storage
🎯 Running text analysis flow with Redis storage...
📊 Final Analysis Summary: {"analysis_complete":true,...}

📈 Example 5: Storage Statistics
📊 Redis Storage Statistics:
  - Total keys: 8
  - Is empty: false
[...]

🎉 Redis storage example completed successfully!
💡 Data persists in Redis - restart the example to see persistence in action!
```