use axum_test::TestServer;
use redb::Database;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tempfile::NamedTempFile;
use futures::future::join_all;

#[derive(Serialize, Deserialize, Debug)]
struct GetMessagesResponse {
    messages: HashMap<String, Vec<String>>,
}

fn create_test_db() -> (Database, NamedTempFile) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::create(temp_file.path()).unwrap();

    let write_txn = db.begin_write().unwrap();
    {
        let _table = write_txn
            .open_table(mailbox_server::MESSAGES_TABLE)
            .unwrap();
    }
    write_txn.commit().unwrap();

    (db, temp_file)
}

fn create_test_server() -> (TestServer, NamedTempFile) {
    let (db, temp_file) = create_test_db();
    let app = mailbox_server::create_app(db);
    let server = TestServer::new(app).unwrap();
    (server, temp_file)
}

#[tokio::test]
async fn stress_test_concurrent_writes() {
    let (server, _temp_file) = create_test_server();
    let server = Arc::new(server);

    let num_concurrent_writes = 100;
    let num_topics = 10;

    let start = Instant::now();
    let mut tasks = Vec::new();

    for i in 0..num_concurrent_writes {
        let server_clone = Arc::clone(&server);
        let topic_id = format!("stress-topic-{}", i % num_topics);
        let message = format!("Message {}", i);

        let task = async move {
            let message_b64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, message.as_bytes());

            let response = server_clone
                .post("/messages/store")
                .json(&json!({
                    "topic_id": topic_id,
                    "message": message_b64
                }))
                .await;

            response.assert_status(axum::http::StatusCode::CREATED);
        };
        tasks.push(task);
    }

    join_all(tasks).await;

    let duration = start.elapsed();

    println!(
        "Concurrent writes: {} messages in {:?} ({:.2} msg/sec)",
        num_concurrent_writes,
        duration,
        num_concurrent_writes as f64 / duration.as_secs_f64()
    );

    // Verify all messages were stored
    for topic_idx in 0..num_topics {
        let topic_id = format!("stress-topic-{}", topic_idx);
        let get_response = server
            .post("/messages/get")
            .json(&json!({
                "topic_ids": [topic_id]
            }))
            .await;

        get_response.assert_status_ok();
        let body: GetMessagesResponse = get_response.json();
        let messages = &body.messages[&format!("stress-topic-{}", topic_idx)];

        assert_eq!(messages.len(), num_concurrent_writes / num_topics);
    }
}

#[tokio::test]
async fn stress_test_concurrent_reads() {
    let (server, _temp_file) = create_test_server();
    let server = Arc::new(server);

    // Pre-populate with messages
    let num_topics = 10;
    let messages_per_topic = 50;

    for topic_idx in 0..num_topics {
        for msg_idx in 0..messages_per_topic {
            let topic_id = format!("read-stress-topic-{}", topic_idx);
            let message = format!("Message {} for topic {}", msg_idx, topic_idx);
            let message_b64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, message.as_bytes());

            server
                .post("/messages/store")
                .json(&json!({
                    "topic_id": topic_id,
                    "message": message_b64
                }))
                .await
                .assert_status(axum::http::StatusCode::CREATED);
        }
    }

    // Now perform concurrent reads
    let num_concurrent_reads = 100;
    let start = Instant::now();
    let mut tasks = Vec::new();

    for i in 0..num_concurrent_reads {
        let server_clone = Arc::clone(&server);
        let topic_id = format!("read-stress-topic-{}", i % num_topics);

        let task = async move {
            let response = server_clone
                .post("/messages/get")
                .json(&json!({
                    "topic_ids": [topic_id]
                }))
                .await;

            response.assert_status_ok();
            let body: GetMessagesResponse = response.json();
            assert_eq!(body.messages[&topic_id].len(), messages_per_topic);
        };
        tasks.push(task);
    }

    join_all(tasks).await;

    let duration = start.elapsed();

    println!(
        "Concurrent reads: {} reads in {:?} ({:.2} reads/sec)",
        num_concurrent_reads,
        duration,
        num_concurrent_reads as f64 / duration.as_secs_f64()
    );
}

#[tokio::test]
async fn stress_test_mixed_read_write_operations() {
    let (server, _temp_file) = create_test_server();
    let server = Arc::new(server);

    let num_operations = 200;
    let num_topics = 10;

    let start = Instant::now();
    let mut tasks: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

    for i in 0..num_operations {
        let server_clone = Arc::clone(&server);
        let topic_id = format!("mixed-topic-{}", i % num_topics);

        if i % 2 == 0 {
            // Write operation
            let task = Box::pin(async move {
                let message = format!("Mixed message {}", i);
                let message_b64 = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    message.as_bytes(),
                );

                let response = server_clone
                    .post("/messages/store")
                    .json(&json!({
                        "topic_id": topic_id,
                        "message": message_b64
                    }))
                    .await;

                response.assert_status(axum::http::StatusCode::CREATED);
            });
            tasks.push(task);
        } else {
            // Read operation
            let task = Box::pin(async move {
                let response = server_clone
                    .post("/messages/get")
                    .json(&json!({
                        "topic_ids": [topic_id]
                    }))
                    .await;

                response.assert_status_ok();
            });
            tasks.push(task);
        }
    }

    join_all(tasks).await;

    let duration = start.elapsed();

    println!(
        "Mixed operations: {} operations in {:?} ({:.2} ops/sec)",
        num_operations,
        duration,
        num_operations as f64 / duration.as_secs_f64()
    );
}

#[tokio::test]
async fn stress_test_large_messages() {
    let (server, _temp_file) = create_test_server();
    let server = Arc::new(server);

    let num_large_messages = 50;
    let message_size = 1024 * 100; // 100KB per message

    let start = Instant::now();
    let mut tasks = Vec::new();

    for i in 0..num_large_messages {
        let server_clone = Arc::clone(&server);
        let topic_id = format!("large-msg-topic-{}", i % 5);

        let task = async move {
            let message = vec![b'X'; message_size];
            let message_b64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &message);

            let response = server_clone
                .post("/messages/store")
                .json(&json!({
                    "topic_id": topic_id,
                    "message": message_b64
                }))
                .await;

            response.assert_status(axum::http::StatusCode::CREATED);
        };
        tasks.push(task);
    }

    join_all(tasks).await;

    let duration = start.elapsed();
    let total_data = num_large_messages * message_size;

    println!(
        "Large messages: {} messages ({} MB) in {:?} ({:.2} MB/sec)",
        num_large_messages,
        total_data / (1024 * 1024),
        duration,
        (total_data as f64 / (1024.0 * 1024.0)) / duration.as_secs_f64()
    );

    // Verify retrieval
    for topic_idx in 0..5 {
        let topic_id = format!("large-msg-topic-{}", topic_idx);
        let response = server
            .post("/messages/get")
            .json(&json!({
                "topic_ids": [topic_id]
            }))
            .await;

        response.assert_status_ok();
        let body: GetMessagesResponse = response.json();
        assert_eq!(
            body.messages[&topic_id].len(),
            num_large_messages / 5
        );
    }
}

#[tokio::test]
async fn stress_test_many_topics() {
    let (server, _temp_file) = create_test_server();

    let num_topics = 1000;
    let messages_per_topic = 5;

    let start = Instant::now();

    for topic_idx in 0..num_topics {
        let topic_id = format!("many-topics-{}", topic_idx);
        for msg_idx in 0..messages_per_topic {
            let message = format!("Message {} for topic {}", msg_idx, topic_idx);
            let message_b64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, message.as_bytes());

            server
                .post("/messages/store")
                .json(&json!({
                    "topic_id": topic_id,
                    "message": message_b64
                }))
                .await
                .assert_status(axum::http::StatusCode::CREATED);
        }
    }

    let store_duration = start.elapsed();

    println!(
        "Many topics: {} topics with {} messages each stored in {:?}",
        num_topics, messages_per_topic, store_duration
    );

    // Test retrieving from multiple topics in one request
    let topic_ids: Vec<String> = (0..100)
        .map(|i| format!("many-topics-{}", i))
        .collect();

    let start = Instant::now();
    let response = server
        .post("/messages/get")
        .json(&json!({
            "topic_ids": topic_ids
        }))
        .await;

    response.assert_status_ok();
    let body: GetMessagesResponse = response.json();

    let retrieve_duration = start.elapsed();

    assert_eq!(body.messages.len(), 100);
    for topic_id in &topic_ids {
        assert_eq!(body.messages[topic_id].len(), messages_per_topic);
    }

    println!(
        "Retrieved 100 topics with {} messages each in {:?}",
        messages_per_topic, retrieve_duration
    );
}

#[tokio::test]
async fn stress_test_rapid_sequential_writes() {
    let (server, _temp_file) = create_test_server();

    let num_messages = 500;
    let topic_id = "rapid-sequential-topic";

    let start = Instant::now();

    for i in 0..num_messages {
        let message = format!("Rapid message {}", i);
        let message_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, message.as_bytes());

        server
            .post("/messages/store")
            .json(&json!({
                "topic_id": topic_id,
                "message": message_b64
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let duration = start.elapsed();

    println!(
        "Rapid sequential writes: {} messages in {:?} ({:.2} msg/sec)",
        num_messages,
        duration,
        num_messages as f64 / duration.as_secs_f64()
    );

    // Verify all messages were stored
    let response = server
        .post("/messages/get")
        .json(&json!({
            "topic_ids": [topic_id]
        }))
        .await;

    response.assert_status_ok();
    let body: GetMessagesResponse = response.json();
    assert_eq!(body.messages[topic_id].len(), num_messages);
}

#[tokio::test]
async fn stress_test_health_endpoint_under_load() {
    let (server, _temp_file) = create_test_server();
    let server = Arc::new(server);

    let num_health_checks = 1000;
    let start = Instant::now();
    let mut tasks = Vec::new();

    for _ in 0..num_health_checks {
        let server_clone = Arc::clone(&server);

        let task = async move {
            let response = server_clone.get("/health").await;
            response.assert_status_ok();
        };
        tasks.push(task);
    }

    join_all(tasks).await;

    let duration = start.elapsed();

    println!(
        "Health checks: {} checks in {:?} ({:.2} checks/sec)",
        num_health_checks,
        duration,
        num_health_checks as f64 / duration.as_secs_f64()
    );
}
