# Rudis Examples

## Basic Usage

Start the server:

```bash
cargo run
```

## Using redis-cli

### Simple Operations

```bash
# Connect to Rudis
redis-cli -p 6379

# Ping server
127.0.0.1:6379> PING
PONG

# Echo message
127.0.0.1:6379> ECHO "Hello"
"Hello"

# Set and get values
127.0.0.1:6379> SET name "Rudis"
OK
127.0.0.1:6379> GET name
"Rudis"

# Check if key exists
127.0.0.1:6379> EXISTS name
(integer) 1

# Delete key
127.0.0.1:6379> DEL name
(integer) 1
```

### Counter Operations

```bash
# Initialize counter
127.0.0.1:6379> SET counter 0
OK

# Increment
127.0.0.1:6379> INCR counter
(integer) 1
127.0.0.1:6379> INCR counter
(integer) 2

# Decrement
127.0.0.1:6379> DECR counter
(integer) 1
```

### List Operations

```bash
# Create a list
127.0.0.1:6379> LPUSH tasks "buy milk"
(integer) 1
127.0.0.1:6379> LPUSH tasks "walk dog" "write code"
(integer) 3

# Add to end
127.0.0.1:6379> RPUSH tasks "sleep"
(integer) 4

# View list
127.0.0.1:6379> LRANGE tasks 0 -1
1) "write code"
2) "walk dog"
3) "buy milk"
4) "sleep"

# Get length
127.0.0.1:6379> LLEN tasks
(integer) 4

# Get by index
127.0.0.1:6379> LINDEX tasks 0
"write code"
127.0.0.1:6379> LINDEX tasks -1
"sleep"

# Remove from head
127.0.0.1:6379> LPOP tasks
"write code"

# Remove from tail
127.0.0.1:6379> RPOP tasks
"sleep"

# View remaining
127.0.0.1:6379> LRANGE tasks 0 -1
1) "walk dog"
2) "buy milk"
```

### Pattern Matching

```bash
# Set multiple keys
127.0.0.1:6379> SET user:1:name "Alice"
OK
127.0.0.1:6379> SET user:2:name "Bob"
OK
127.0.0.1:6379> SET product:1 "Laptop"
OK

# Find all keys
127.0.0.1:6379> KEYS *
1) "user:1:name"
2) "user:2:name"
3) "product:1"

# Find keys with pattern
127.0.0.1:6379> KEYS user:*
1) "user:1:name"
2) "user:2:name"
```

### Expiration

```bash
# Set key with 60 second expiration
127.0.0.1:6379> SET session:abc "user_data" EX 60
OK

# Set key with 5000 millisecond expiration
127.0.0.1:6379> SET cache:key "cached_value" PX 5000
OK

# Update expiration on existing key
127.0.0.1:6379> SET mykey "value"
OK
127.0.0.1:6379> EXPIRE mykey 30
(integer) 1
```

### Database Operations

```bash
# Check database size
127.0.0.1:6379> DBSIZE
(integer) 5

# Clear all keys
127.0.0.1:6379> FLUSHDB
OK
127.0.0.1:6379> DBSIZE
(integer) 0
```

## Using from Code

### Rust

```rust
use std::net::TcpStream;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6379")?;

    // SET command: *3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n
    let cmd = b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
    stream.write_all(cmd)?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;
    println!("Response: {:?}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}
```

### Go

```go
package main

import (
    "fmt"
    "net"
)

func sendCommand(conn net.Conn, args ...string) (string, error) {
    command := fmt.Sprintf("*%d\r\n", len(args))
    for _, arg := range args {
        command += fmt.Sprintf("$%d\r\n%s\r\n", len(arg), arg)
    }
    _, err := conn.Write([]byte(command))
    if err != nil {
        return "", err
    }

    buffer := make([]byte, 1024)
    n, err := conn.Read(buffer)
    if err != nil {
        return "", err
    }
    return string(buffer[:n]), nil
}

func main() {
    conn, err := net.Dial("tcp", "127.0.0.1:6379")
    if err != nil {
        panic(err)
    }
    defer conn.Close()

    // Example usage
    response, err := sendCommand(conn, "PING")
    if err != nil {
        panic(err)
    }
    fmt.Println("Response:", response)
}
```

### Python

```python
import socket

def send_command(sock, *args):
    """Send a Redis command using RESP protocol"""
    command = f"*{len(args)}\r\n"
    for arg in args:
        arg_bytes = str(arg).encode()
        command += f"${len(arg_bytes)}\r\n{arg_bytes.decode()}\r\n"
    sock.send(command.encode())
    return sock.recv(1024).decode()

# Connect to Rudis
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.connect(('127.0.0.1', 6379))

# Execute commands
print(send_command(sock, 'PING'))
print(send_command(sock, 'SET', 'mykey', 'myvalue'))
print(send_command(sock, 'GET', 'mykey'))

sock.close()
```

### Node.js

```javascript
const net = require('net');

const client = net.createConnection({ port: 6379 }, () => {
    console.log('Connected to Rudis');

    // Send PING command
    const cmd = '*1\r\n$4\r\nPING\r\n';
    client.write(cmd);
});

client.on('data', (data) => {
    console.log('Response:', data.toString());
    client.end();
});

client.on('end', () => {
    console.log('Disconnected');
});
```

## Performance Testing

### Using redis-benchmark

```bash
# Install redis-tools if needed
# brew install redis (macOS)
# sudo apt-get install redis-tools (Ubuntu)

# Run benchmark
redis-benchmark -h 127.0.0.1 -p 6379 -t set,get -n 100000 -q

# Expected output:
# SET: ~50000-100000 requests per second
# GET: ~50000-100000 requests per second
```

## Environment Variables

```bash
# Change server address
RUDIS_ADDR=0.0.0.0:6379 cargo run

# Run on different port
RUDIS_ADDR=127.0.0.1:7000 cargo run
```
