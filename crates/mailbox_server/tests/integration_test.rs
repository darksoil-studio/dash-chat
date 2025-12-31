use axum_test::TestServer;
use redb::Database;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize, Debug)]
struct GetMessagesResponse {
    messages: HashMap<String, Vec<String>>,
}

fn create_test_db() -> (Database, NamedTempFile) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::create(temp_file.path()).unwrap();

    let write_txn = db.begin_write().unwrap();
    {
        let _table = write_txn.open_table(mailbox_server::MESSAGES_TABLE).unwrap();
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
async fn test_health_check() {
    let (server, _temp_file) = create_test_server();

    let response = server.get("/health").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "status": "ok"
    }));
}

#[tokio::test]
async fn test_store_and_retrieve_single_message() {
    let (server, _temp_file) = create_test_server();

    let message_data = b"Hello, World!";
    let message_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, message_data);

    let store_response = server
        .post("/messages/store")
        .json(&json!({
            "topic_id": "test-topic-1",
            "message": message_b64
        }))
        .await;

    store_response.assert_status(axum::http::StatusCode::CREATED);

    let get_response = server
        .post("/messages/get")
        .json(&json!({
            "topic_ids": ["test-topic-1"]
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetMessagesResponse = get_response.json();
    assert!(body.messages.contains_key("test-topic-1"));

    let topic_messages = &body.messages["test-topic-1"];
    assert_eq!(topic_messages.len(), 1);

    let retrieved_message = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &topic_messages[0]
    ).unwrap();
    assert_eq!(retrieved_message, message_data);
}

#[tokio::test]
async fn test_store_and_retrieve_multiple_messages_same_topic() {
    let (server, _temp_file) = create_test_server();

    let messages = vec![
        b"First message".to_vec(),
        b"Second message".to_vec(),
        b"Third message".to_vec(),
    ];

    for message in &messages {
        let message_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, message);
        server
            .post("/messages/store")
            .json(&json!({
                "topic_id": "test-topic-multi",
                "message": message_b64
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let get_response = server
        .post("/messages/get")
        .json(&json!({
            "topic_ids": ["test-topic-multi"]
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetMessagesResponse = get_response.json();
    let topic_messages = &body.messages["test-topic-multi"];

    assert_eq!(topic_messages.len(), 3);

    for (i, msg_b64) in topic_messages.iter().enumerate() {
        let retrieved = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            msg_b64
        ).unwrap();
        assert_eq!(retrieved, messages[i]);
    }
}

#[tokio::test]
async fn test_retrieve_messages_from_multiple_topics() {
    let (server, _temp_file) = create_test_server();

    let topic1_msg = b"Topic 1 message";
    let topic2_msg = b"Topic 2 message";

    server
        .post("/messages/store")
        .json(&json!({
            "topic_id": "topic-a",
            "message": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, topic1_msg)
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    server
        .post("/messages/store")
        .json(&json!({
            "topic_id": "topic-b",
            "message": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, topic2_msg)
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let get_response = server
        .post("/messages/get")
        .json(&json!({
            "topic_ids": ["topic-a", "topic-b"]
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetMessagesResponse = get_response.json();
    let messages = &body.messages;

    assert!(messages.contains_key("topic-a"));
    assert!(messages.contains_key("topic-b"));

    let topic_a_messages = &messages["topic-a"];
    let topic_b_messages = &messages["topic-b"];

    assert_eq!(topic_a_messages.len(), 1);
    assert_eq!(topic_b_messages.len(), 1);

    let retrieved_a = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &topic_a_messages[0]
    ).unwrap();
    let retrieved_b = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &topic_b_messages[0]
    ).unwrap();

    assert_eq!(retrieved_a, topic1_msg);
    assert_eq!(retrieved_b, topic2_msg);
}

#[tokio::test]
async fn test_retrieve_empty_topic() {
    let (server, _temp_file) = create_test_server();

    let get_response = server
        .post("/messages/get")
        .json(&json!({
            "topic_ids": ["non-existent-topic"]
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetMessagesResponse = get_response.json();
    let messages = &body.messages;

    assert!(messages.contains_key("non-existent-topic"));
    assert_eq!(messages["non-existent-topic"].len(), 0);
}

#[tokio::test]
async fn test_topic_isolation() {
    let (server, _temp_file) = create_test_server();

    server
        .post("/messages/store")
        .json(&json!({
            "topic_id": "isolated-topic-1",
            "message": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 1")
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    server
        .post("/messages/store")
        .json(&json!({
            "topic_id": "isolated-topic-2",
            "message": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 2")
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let get_response = server
        .post("/messages/get")
        .json(&json!({
            "topic_ids": ["isolated-topic-1"]
        }))
        .await;

    let body: GetMessagesResponse = get_response.json();
    let messages = &body.messages;

    assert_eq!(messages["isolated-topic-1"].len(), 1);
    assert!(!messages.contains_key("isolated-topic-2"));
}
