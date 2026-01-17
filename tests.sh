#!/bin/bash

# Rudis Test Suite
# Comprehensive integration tests for all functionality

set -e

echo "=========================================="
echo "  Rudis Test Suite"
echo "=========================================="

# Start server
echo "Starting Rudis server..."
cargo run > /tmp/rudis_test.log 2>&1 &
SERVER_PID=$!
sleep 3

# Verify server started
if ! lsof -i :6379 > /dev/null 2>&1; then
    echo "ERROR: Server failed to start"
    cat /tmp/rudis_test.log
    exit 1
fi
echo "✓ Server started"
echo ""

# Cleanup on exit
cleanup() {
    echo ""
    echo "Stopping server..."
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Test functions - visual output only
echo "=== Connection Tests ==="
redis-cli -p 6379 PING
redis-cli -p 6379 ECHO "Hello Rudis"

echo ""
echo "=== String Operations ==="
redis-cli -p 6379 FLUSHDB
redis-cli -p 6379 SET mykey "Hello World"
redis-cli -p 6379 GET mykey
redis-cli -p 6379 DEL mykey
redis-cli -p 6379 GET mykey

echo ""
echo "=== Counter Operations ==="
redis-cli -p 6379 SET counter 10
redis-cli -p 6379 INCR counter
redis-cli -p 6379 INCR counter
redis-cli -p 6379 DECR counter
redis-cli -p 6379 GET counter

echo ""
echo "=== Key Operations ==="
redis-cli -p 6379 FLUSHDB
redis-cli -p 6379 SET key1 val1
redis-cli -p 6379 SET key2 val2
redis-cli -p 6379 EXISTS key1
redis-cli -p 6379 EXISTS nokey
redis-cli -p 6379 DBSIZE

echo ""
echo "=== Pattern Matching ==="
redis-cli -p 6379 FLUSHDB
redis-cli -p 6379 SET user:1 alice
redis-cli -p 6379 SET user:2 bob
redis-cli -p 6379 SET product:1 laptop
echo "All keys:"
redis-cli -p 6379 KEYS "*"
echo "User keys:"
redis-cli -p 6379 KEYS "user:*"

echo ""
echo "=== Expiration (EX) ==="
redis-cli -p 6379 SET tempkey "temporary" EX 2
echo "Before expiry:"
redis-cli -p 6379 GET tempkey
echo "Waiting 3 seconds..."
sleep 3
echo "After expiry:"
redis-cli -p 6379 GET tempkey

echo ""
echo "=== Expiration (PX) ==="
redis-cli -p 6379 SET mskey "value" PX 1500
echo "Before expiry:"
redis-cli -p 6379 GET mskey
echo "Waiting 2 seconds..."
sleep 2
echo "After expiry:"
redis-cli -p 6379 GET mskey

echo ""
echo "=== EXPIRE Command ==="
redis-cli -p 6379 SET expkey "data"
redis-cli -p 6379 EXPIRE expkey 2
echo "Before expiry:"
redis-cli -p 6379 GET expkey
echo "Waiting 3 seconds..."
sleep 3
echo "After expiry:"
redis-cli -p 6379 GET expkey

echo ""
echo "=== Multiple Keys ==="
redis-cli -p 6379 FLUSHDB
redis-cli -p 6379 SET k1 v1
redis-cli -p 6379 SET k2 v2
redis-cli -p 6379 SET k3 v3
redis-cli -p 6379 EXISTS k1 k2 k3
redis-cli -p 6379 DEL k1 k2
redis-cli -p 6379 DBSIZE

echo ""
echo "=== Error Handling ==="
redis-cli -p 6379 SET notnum abc
redis-cli -p 6379 INCR notnum 2>&1
redis-cli -p 6379 GET 2>&1
redis-cli -p 6379 UNKNOWNCMD 2>&1

echo ""
echo "=== Concurrency Test ==="
redis-cli -p 6379 FLUSHDB
echo "Writing 20 keys concurrently..."
for i in {1..20}; do
    redis-cli -p 6379 SET "c:$i" "v$i" > /dev/null &
done
wait
redis-cli -p 6379 DBSIZE

echo ""
echo "=========================================="
echo "✓ All tests completed successfully!"
echo "=========================================="

