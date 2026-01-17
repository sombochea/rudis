# Rudis - Redis in Rust

A Redis-like in-memory data store implementation in Rust, built from scratch.

## Features

- **RESP Protocol**: Full implementation of Redis Serialization Protocol (RESP)
- **TCP Server**: Async TCP server using Tokio
- **In-Memory Store**: Thread-safe key-value storage with RwLock
- **TTL Support**: Keys can expire after a specified time
- **Multiple Data Operations**: Support for strings, integers, and basic operations

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

## Building and Running

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run
```

Or specify a custom address:
```bash
RUDIS_ADDR=0.0.0.0:6379 cargo run
```

## Testing with redis-cli

You can test Rudis using the standard Redis CLI:

```bash
redis-cli -p 6379

# Basic operations
127.0.0.1:6379> PING
PONG
127.0.0.1:6379> SET mykey "Hello"
OK
127.0.0.1:6379> GET mykey
"Hello"
127.0.0.1:6379> SET counter 10
OK
127.0.0.1:6379> INCR counter
(integer) 11
127.0.0.1:6379> DEL mykey
(integer) 1
127.0.0.1:6379> EXISTS mykey
(integer) 0
127.0.0.1:6379> KEYS *
1) "counter"

# TTL operations
127.0.0.1:6379> SET session "data" EX 60
OK
127.0.0.1:6379> SET cache "value" PX 5000
OK
```

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

## Future Enhancements

- [ ] Persistence (RDB snapshots, AOF)
- [ ] Replication (master-slave)
- [ ] Pub/Sub messaging
- [ ] Lua scripting
- [ ] Transactions (MULTI/EXEC)
- [ ] Lists, Sets, Sorted Sets, Hashes
- [ ] Cluster mode
- [ ] Benchmarking tools

## License

MIT
