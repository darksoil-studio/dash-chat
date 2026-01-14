# Mailbox Server

A simple HTTP server for storing and retrieving messages organized by topics. Built with Axum and redb.

## Features

- Store messages to topics via HTTP API
- Retrieve messages from one or more topics
- Persistent storage using redb (embedded database)
- Messages are stored with UUID v7 keys for time-ordered retrieval
- CORS enabled for cross-origin requests

## Installation

```bash
cargo build --release
```

## Usage

### Starting the Server

**Default configuration:**
```bash
cargo run --bin mailbox_server
```

This starts the server on `0.0.0.0:3000` with database file `mailbox.redb`.

**Custom configuration:**
```bash
cargo run --bin mailbox_server -- --db-path /path/to/database.redb --addr 127.0.0.1:8080
```

**Command-line options:**
- `-d, --db-path <DB_PATH>`: Path to the redb database file (default: `mailbox.redb`)
- `-a, --addr <ADDR>`: Address to bind the server to (default: `0.0.0.0:3000`)
- `-h, --help`: Print help information

### API Endpoints

#### Health Check
```bash
GET /health
```

Response:
```json
{
  "status": "ok"
}
```

#### Store a Message
```bash
POST /messages/store
Content-Type: application/json

{
  "topic_id": "my-topic",
  "message": "SGVsbG8sIFdvcmxkIQ=="  // base64-encoded message
}
```

Response: `201 Created`

#### Retrieve Messages
```bash
POST /messages/get
Content-Type: application/json

{
  "topic_ids": ["my-topic", "another-topic"]
}
```

Response:
```json
{
  "messages": {
    "my-topic": ["SGVsbG8sIFdvcmxkIQ==", "QW5vdGhlciBtZXNzYWdl"],
    "another-topic": []
  }
}
```

### Example Usage with curl

**Store a message:**
```bash
curl -X POST http://localhost:3000/messages/store \
  -H "Content-Type: application/json" \
  -d '{
    "topic_id": "test-topic",
    "message": "SGVsbG8sIFdvcmxkIQ=="
  }'
```

**Retrieve messages:**
```bash
curl -X POST http://localhost:3000/messages/get \
  -H "Content-Type: application/json" \
  -d '{
    "topic_ids": ["test-topic"]
  }'
```

## Development

### Running Tests

**Integration tests:**
```bash
cargo test --test integration_test
```

**Stress tests:**
```bash
cargo test --test stress_test -- --nocapture
```

The stress tests include:
- Concurrent writes and reads
- Mixed read/write operations
- Large message handling
- Many topics scalability
- Health endpoint load testing

### Environment Variables

- `RUST_LOG`: Set logging level (e.g., `RUST_LOG=debug`)

Example:
```bash
RUST_LOG=mailbox_server=debug,tower_http=debug cargo run --bin mailbox_server
```

## Architecture

- **Storage**: redb (embedded key-value database)
- **Web framework**: Axum
- **Message format**: Binary data encoded as base64 in JSON
- **Key format**: `{topic_id}:{uuid_v7}` for time-ordered retrieval
- **Concurrency**: Async/await with Tokio runtime

## License

See the root project license.
