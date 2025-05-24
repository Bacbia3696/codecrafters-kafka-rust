# Graceful Shutdown Implementation

This document explains the graceful shutdown capabilities added to the Kafka broker implementation.

## Overview

The graceful shutdown implementation ensures that the Kafka broker can be stopped cleanly without abruptly terminating active connections. This is crucial for production environments where maintaining data integrity and connection stability is important.

## Features

### 🛡️ Signal Handling
- **SIGINT (Ctrl+C)**: Gracefully shuts down the server
- **SIGTERM**: Gracefully shuts down the server (Unix/Linux)
- Cross-platform support (Windows uses SIGINT only)

### 🔄 Connection Management
- **Concurrent Connections**: Each client connection runs in its own async task
- **Graceful Termination**: Active connections are allowed to finish their current operations
- **Timeout Protection**: Connections have a 5-minute timeout to prevent hanging
- **Coordinated Shutdown**: All connections receive shutdown signals simultaneously

### ⏱️ Shutdown Coordination
- **Broadcast Channels**: Used to coordinate shutdown across all tasks
- **30-Second Grace Period**: Server waits up to 30 seconds for connections to finish
- **Clean Exit**: Proper cleanup and resource deallocation

## Architecture

### Key Components

```rust
// Shutdown coordination primitives
let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
let active_connections = Arc<Notify::new();

// Signal handling task
tokio::spawn(async move {
    wait_for_shutdown_signal().await;
    shutdown_tx.send(()).unwrap();
});

// Main server loop with shutdown awareness
tokio::select! {
    accept_result = listener.accept() => {
        // Handle new connections
    }
    _ = shutdown_rx.recv() => {
        // Initiate shutdown
        break;
    }
}
```

### Connection Handling

Each connection is handled in a separate async task:

```rust
tokio::spawn(async move {
    let result = tokio::select! {
        // Normal connection handling
        handle_result = handle_connection_with_timeout(&broker, stream, peer_addr) => {
            handle_result
        }
        // Shutdown signal received
        _ = connection_shutdown.recv() => {
            println!("Connection {} shutting down due to server shutdown", peer_addr);
            Ok(())
        }
    };
});
```

## Usage

### Basic Server Start
```rust
use codecrafters_kafka::{KafkaBroker, NetworkServer};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:9092".parse()?;
    let broker = KafkaBroker::new();
    let server = NetworkServer::new(broker);
    
    // This will handle graceful shutdown automatically
    server.start(addr).await
}
```

### Alternative API (Future Extension)
```rust
// Future convenience methods
let server = NetworkServer::with_graceful_shutdown(broker);
server.run(addr).await  // Equivalent to start() but more semantic
```

## Testing

### Automated Tests

Run the comprehensive test suite:

```bash
# Test basic functionality
python3 test_graceful_shutdown.py

# Test graceful shutdown with active connections
python3 test_graceful_shutdown.py --graceful-shutdown
```

### Manual Testing

1. **Start the broker:**
   ```bash
   cargo run
   ```

2. **Connect multiple clients:**
   ```bash
   python3 test_client.py  # In separate terminals
   ```

3. **Trigger graceful shutdown:**
   ```bash
   # Press Ctrl+C in the broker terminal
   # OR send SIGTERM: kill -TERM <broker_pid>
   ```

4. **Observe the output:**
   ```
   Received SIGINT
   Shutdown signal received, initiating graceful shutdown...
   Server shutdown initiated...
   Waiting for active connections to finish...
   All connections finished gracefully
   Kafka broker shutdown complete
   ```

## Implementation Details

### Proper Message Parsing

The implementation correctly handles the Kafka wire protocol:

1. **Message Length Prefix**: Reads 4-byte message length first
2. **Request Parsing**: Parses RequestHeaderV2 with proper NULLABLE_STRING handling
3. **Error Handling**: Comprehensive error handling with debugging information

### Request Processing Flow

```
1. Read 4-byte message length
2. Read message content (length bytes)
3. Parse RequestHeaderV2:
   - API Key (2 bytes)
   - API Version (2 bytes) 
   - Correlation ID (4 bytes)
   - Client ID (NULLABLE_STRING)
   - Tagged fields (1 byte, empty)
4. Generate appropriate response
```

### Debug Features

The implementation includes comprehensive debugging:

- **Hex Dumps**: Shows raw message bytes for debugging
- **Field-by-Field Parsing**: Detailed breakdown of protocol fields
- **Connection Tracking**: Logs connection lifecycle events
- **Error Context**: Rich error messages with expected vs actual values

## Example Output

### Successful Request Processing
```
Accepted connection from 127.0.0.1:61325
Handling new client connection
Expected message length: 31 bytes
Parsing request buffer of 31 bytes
Request Buffer: 31 bytes
0000: 00 12 00 01 00 00 00 01  00 14 73 68 75 74 64 6F  |..........shutdo|
0010: 77 6E 2D 74 65 73 74 2D  63 6C 69 65 6E 74 00     |wn-test-client.|

Successfully parsed request header:
  API Key: 18
  API Version: 1
  Correlation ID: 1
  Client ID: 'shutdown-test-client'
```

### Graceful Shutdown Sequence
```
Received SIGINT
Shutdown signal received, initiating graceful shutdown...
Server shutdown initiated...
Waiting for active connections to finish...
All connections finished gracefully
Kafka broker shutdown complete
```

## Error Handling

The implementation handles various error scenarios:

- **Insufficient bytes**: When message is shorter than expected
- **Invalid UTF-8**: When client_id contains invalid UTF-8
- **Connection timeouts**: 5-minute timeout per connection
- **Signal handling errors**: Fallback mechanisms for signal setup failures

## Future Enhancements

1. **Connection Tracking**: More sophisticated active connection counting
2. **Partial Message Handling**: Better support for fragmented messages  
3. **Health Checks**: Pre-shutdown health verification
4. **Configurable Timeouts**: Adjustable grace periods
5. **Metrics**: Shutdown performance and connection statistics

## SOLID Principles Adherence

- **Single Responsibility**: NetworkServer handles only network concerns
- **Open/Closed**: Extensible via traits without modifying existing code
- **Liskov Substitution**: KafkaBroker can be substituted with other implementations
- **Interface Segregation**: Small, focused interfaces
- **Dependency Inversion**: Depends on abstractions (traits), not concretions

This implementation provides a robust foundation for production-ready Kafka broker operations with proper shutdown semantics. 