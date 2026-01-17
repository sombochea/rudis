# Rudis - Redis in Rust

A Redis-like in-memory data store implementation in Rust, built from scratch.

## Features

- **RESP Protocol**: Full implementation of Redis Serialization Protocol (RESP)
- **TCP Server**: Async TCP server using Tokio
- **In-Memory Store**: Thread-safe key-value storage with RwLock
- **TTL Support**: Keys can expire after a specified time (seconds/milliseconds)
- **Concurrency**: Handles 20+ simultaneous connections
- **Error Handling**: Redis-compatible error messages

## Supported Commands

### Connection
- `PING [message]` - Ping the server
- `ECHO message` - Echo the given string

### String Operations
- `GET key` - Get the value of a key
- `SET key value [EX seconds] [PX milliseconds]` - Set the string value of a key with optional expiry
- `DEL key [key ...]` - Delete one or more keys
- `EXISTS key [key ...]` - Check if keys exist
- `EXPIRE key seconds` - Set a key's time to live in seconds

### Numeric Operations
- `INCR key` - Increment the integer value of a key by one
- `DECR key` - Decrement the integer value of a key by one

### Server Operations
- `KEYS pattern` - Find all keys matching the given pattern
- `DBSIZE` - Return the number of keys in the database
- `FLUSHDB` - Remove all keys from the current database

## Quick Start

### Build and Run
```bash
# Development mode
cargo run

# Production mode
cargo build --release
./target/release/rudis

# Custom address
RUDIS_ADDR=0.0.0.0:6379 cargo run
```

### Run Tests
```bash
./tests.sh
```

## Usage

Connect using standard Redis CLI:

```bash
redis-cli -p 6379

127.0.0.1:6379> PING
PONG
127.0.0.1:6379> SET mykey "Hello"
OK
127.0.0.1:6379> GET mykey
"Hello"
127.0.0.1:6379> INCR counter
(integer) 1
127.0.0.1:6379> KEYS *
1) "mykey"
2) "counter"
```

See [EXAMPLES.md](EXAMPLES.md) for more usage examples.

## Architecture

### Components

1. **RESP Parser** (`resp.rs`)
   - Parses Redis Serialization Protocol
   - Supports all RESP data types: Simple Strings, Errors, Integers, Bulk Strings, Arrays
   - Serializes responses back to RESP format

2. **Store** (`store.rs`)
   - Thread-safe in-memory HashMap with RwLock
   - Supports key expiration with TTL
   - Automatic cleanup of expired keys on access

3. **Command Handler** (`command.rs`)
   - Parses commands from RESP arrays
   - Executes commands against the store
   - Returns properly formatted RESP responses

4. **TCP Server** (`server.rs`)
   - Async TCP server using Tokio
   - Handles multiple concurrent connections
   - Spawns a new task for each client connection

## Implementation Details

- **Concurrency**: Uses `Arc<RwLock<HashMap>>` for thread-safe shared state
- **Async I/O**: Built on Tokio for efficient async operations
- **Memory Safety**: Leverages Rust's ownership system for safety guarantees
- **Zero-Copy**: Uses `Vec<u8>` for binary data to avoid unnecessary allocations

## Roadmap

### Planned Features
- [ ] Lists (LPUSH, RPUSH, LPOP, RPOP, LRANGE)
- [ ] Sets (SADD, SREM, SMEMBERS, SISMEMBER)
- [ ] Sorted Sets (ZADD, ZRANGE, ZREM)
- [ ] Hashes (HSET, HGET, HDEL, HGETALL)
- [ ] Persistence (RDB snapshots, AOF)
- [ ] Pub/Sub messaging
- [ ] Transactions (MULTI/EXEC)
- [ ] Replication (master-slave)

## License

MIT
