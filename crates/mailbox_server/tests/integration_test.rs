use axum_test::TestServer;
use mailbox_server::GetBlobsResponse;
use redb::Database;
use serde_json::json;
use tempfile::NamedTempFile;

fn create_test_db() -> (Database, NamedTempFile) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::create(temp_file.path()).unwrap();

    let write_txn = db.begin_write().unwrap();
    {
        let _table = write_txn.open_table(mailbox_server::BLOBS_TABLE).unwrap();
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
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "test-topic-1": {
                    "log-a": {
                        "0": message_b64
                    }
                }
            }
        }))
        .await;

    store_response.assert_status(axum::http::StatusCode::CREATED);

    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "test-topic-1": {}
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    assert!(body.blobs_by_topic.contains_key("test-topic-1"));

    let topic_response = &body.blobs_by_topic["test-topic-1"];
    assert!(topic_response.blobs.contains_key("log-a"));
    assert!(topic_response.missing.is_empty());

    let log_sequences = &topic_response.blobs["log-a"];
    assert!(log_sequences.contains_key(&0));

    let retrieved_message = &log_sequences[&0];
    assert_eq!(retrieved_message.as_ref(), message_data);
}

#[tokio::test]
async fn test_store_and_retrieve_multiple_messages_same_topic() {
    let (server, _temp_file) = create_test_server();

    server
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "test-topic-multi": {
                    "log-1": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"First message".to_vec()),
                        "1": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Second message".to_vec()),
                        "2": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Third message".to_vec()),
                    }
                }
            }
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "test-topic-multi": {}
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let topic_response = &body.blobs_by_topic["test-topic-multi"];
    let log_sequences = &topic_response.blobs["log-1"];

    assert_eq!(log_sequences.len(), 3);
    assert!(topic_response.missing.is_empty());

    assert_eq!(log_sequences[&(0 as u32)].as_ref(), b"First message");
    assert_eq!(log_sequences[&(1 as u32)].as_ref(), b"Second message");
    assert_eq!(log_sequences[&(2 as u32)].as_ref(), b"Third message");
}

#[tokio::test]
async fn test_retrieve_messages_from_multiple_topics() {
    let (server, _temp_file) = create_test_server();

    let topic1_msg = b"Topic 1 message";
    let topic2_msg = b"Topic 2 message";

    server
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "topic-a": {
                    "log-1": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, topic1_msg)
                    }
                },
                "topic-b": {
                    "log-1": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, topic2_msg)
                    }
                }
            }
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "topic-a": {},
                "topic-b": {}
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let blobs_for_topics = &body.blobs_by_topic;

    assert!(blobs_for_topics.contains_key("topic-a"));
    assert!(blobs_for_topics.contains_key("topic-b"));

    let topic_a_response = &blobs_for_topics["topic-a"];
    let topic_b_response = &blobs_for_topics["topic-b"];

    assert_eq!(topic_a_response.blobs["log-1"].len(), 1);
    assert_eq!(topic_b_response.blobs["log-1"].len(), 1);
    assert!(topic_a_response.missing.is_empty());
    assert!(topic_b_response.missing.is_empty());

    let retrieved_a = &topic_a_response.blobs["log-1"][&0];
    let retrieved_b = &topic_b_response.blobs["log-1"][&0];

    assert_eq!(retrieved_a.as_ref(), topic1_msg);
    assert_eq!(retrieved_b.as_ref(), topic2_msg);
}

#[tokio::test]
async fn test_retrieve_empty_topic() {
    let (server, _temp_file) = create_test_server();

    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "non-existent-topic": {}
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let blobs_for_topics = &body.blobs_by_topic;

    assert!(blobs_for_topics.contains_key("non-existent-topic"));
    let topic_response = &blobs_for_topics["non-existent-topic"];
    assert_eq!(topic_response.blobs.len(), 0);
    assert!(topic_response.missing.is_empty());
}

#[tokio::test]
async fn test_topic_isolation() {
    let (server, _temp_file) = create_test_server();

    server
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "isolated-topic-1": {
                    "log-1": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 1")
                    }
                },
                "isolated-topic-2": {
                    "log-1": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 2")
                    }
                }
            }
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "isolated-topic-1": {}
            }
        }))
        .await;

    let body: GetBlobsResponse = get_response.json();
    let blobs_for_topics = &body.blobs_by_topic;

    let topic_1_response = &blobs_for_topics["isolated-topic-1"];
    assert_eq!(topic_1_response.blobs["log-1"].len(), 1);
    assert!(topic_1_response.missing.is_empty());
    assert!(!blobs_for_topics.contains_key("isolated-topic-2"));
}

#[tokio::test]
async fn test_sequence_number_filtering() {
    let (server, _temp_file) = create_test_server();

    // Store messages with sequence numbers 0, 1, 2, 3, 4
    server
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "test-topic": {
                    "log-x": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 0"),
                        "1": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 1"),
                        "2": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 2"),
                        "3": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 3"),
                        "4": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 4")
                    }
                }
            }
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Request only messages with sequence number > 2
    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "test-topic": {
                    "log-x": 2
                }
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let topic_response = &body.blobs_by_topic["test-topic"];
    let log_sequences = &topic_response.blobs["log-x"];

    // Should only get messages 3 and 4
    assert_eq!(log_sequences.len(), 2);
    assert!(log_sequences.contains_key(&3));
    assert!(log_sequences.contains_key(&4));
    assert!(!log_sequences.contains_key(&2));
    assert!(!log_sequences.contains_key(&1));
    assert!(!log_sequences.contains_key(&0));

    // Server has all messages up to 4, client asked for > 2, so no missing
    assert!(topic_response.missing.is_empty());

    let msg3 = &log_sequences[&3];
    let msg4 = &log_sequences[&4];

    assert_eq!(msg3.as_ref(), b"Message 3");
    assert_eq!(msg4.as_ref(), b"Message 4");
}

#[tokio::test]
async fn test_get_returns_all_logs_for_topic() {
    let (server, _temp_file) = create_test_server();

    // Store messages in multiple logs for the same topic
    server
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "test-topic": {
                    "log-a": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Log A - Message 0"),
                        "1": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Log A - Message 1"),
                        "2": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Log A - Message 2")
                    },
                    "log-b": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Log B - Message 0"),
                        "1": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Log B - Message 1")
                    },
                    "log-c": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Log C - Message 0")
                    }
                }
            }
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Request only log-a with sequence > 0, but should also get all of log-b and log-c
    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "test-topic": {
                    "log-a": 0
                }
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let topic_response = &body.blobs_by_topic["test-topic"];
    let topic_logs = &topic_response.blobs;

    // Should have all three logs
    assert_eq!(topic_logs.len(), 3);
    assert!(topic_logs.contains_key("log-a"));
    assert!(topic_logs.contains_key("log-b"));
    assert!(topic_logs.contains_key("log-c"));

    // log-a should only have messages with seq > 0 (messages 1 and 2)
    let log_a = &topic_logs["log-a"];
    assert_eq!(log_a.len(), 2);
    assert!(log_a.contains_key(&1));
    assert!(log_a.contains_key(&2));
    assert!(!log_a.contains_key(&0));

    // log-b should have ALL messages (was not in request)
    let log_b = &topic_logs["log-b"];
    assert_eq!(log_b.len(), 2);
    assert!(log_b.contains_key(&0));
    assert!(log_b.contains_key(&1));

    // log-c should have ALL messages (was not in request)
    let log_c = &topic_logs["log-c"];
    assert_eq!(log_c.len(), 1);
    assert!(log_c.contains_key(&0));

    // No missing since server has all messages
    assert!(topic_response.missing.is_empty());

    // Verify content
    let msg_a1 = &log_a[&1];
    assert_eq!(msg_a1.as_ref(), b"Log A - Message 1");

    let msg_b0 = &log_b[&0];
    assert_eq!(msg_b0.as_ref(), b"Log B - Message 0");
}

#[tokio::test]
async fn test_missing_blobs_server_behind() {
    let (server, _temp_file) = create_test_server();

    // Store only messages 0-2 on server
    server
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "test-topic": {
                    "log-x": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 0"),
                        "1": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 1"),
                        "2": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 2")
                    }
                }
            }
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Client says it has up to sequence 5
    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "test-topic": {
                    "log-x": 5
                }
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let topic_response = &body.blobs_by_topic["test-topic"];

    // Server should return empty blobs (nothing new for client)
    // The log might not even exist in blobs if all were filtered out
    assert!(topic_response.blobs.get("log-x").map_or(true, |log| log.is_empty()));

    // Server should report missing sequences 3, 4, 5
    assert!(topic_response.missing.contains_key("log-x"));
    let missing_seqs = &topic_response.missing["log-x"];
    assert_eq!(missing_seqs.len(), 3);
    assert_eq!(missing_seqs, &vec![3, 4, 5]);
}

#[tokio::test]
async fn test_missing_blobs_server_has_nothing() {
    let (server, _temp_file) = create_test_server();

    // Don't store any blobs

    // Client says it has up to sequence 3
    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "test-topic": {
                    "log-x": 3
                }
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let topic_response = &body.blobs_by_topic["test-topic"];

    // Server should return empty blobs
    assert!(topic_response.blobs.is_empty());

    // Server should report missing all sequences 0-3
    assert!(topic_response.missing.contains_key("log-x"));
    let missing_seqs = &topic_response.missing["log-x"];
    assert_eq!(missing_seqs.len(), 4);
    assert_eq!(missing_seqs, &vec![0, 1, 2, 3]);
}

#[tokio::test]
async fn test_missing_blobs_multiple_logs() {
    let (server, _temp_file) = create_test_server();

    // Store blobs for log-a (0-1) but nothing for log-b
    server
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "test-topic": {
                    "log-a": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Log A - 0"),
                        "1": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Log A - 1")
                    }
                }
            }
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Client says it has log-a up to 4 and log-b up to 2
    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "test-topic": {
                    "log-a": 4,
                    "log-b": 2
                }
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let topic_response = &body.blobs_by_topic["test-topic"];

    // Should have missing for both logs
    assert_eq!(topic_response.missing.len(), 2);

    // log-a: server has 0-1, client has 0-4, missing 2-4
    let missing_log_a = &topic_response.missing["log-a"];
    assert_eq!(missing_log_a, &vec![2, 3, 4]);

    // log-b: server has nothing, client has 0-2, missing all
    let missing_log_b = &topic_response.missing["log-b"];
    assert_eq!(missing_log_b, &vec![0, 1, 2]);
}

#[tokio::test]
async fn test_no_missing_when_server_is_ahead() {
    let (server, _temp_file) = create_test_server();

    // Store messages 0-5
    server
        .post("/blobs/store")
        .json(&json!({
            "blobs": {
                "test-topic": {
                    "log-x": {
                        "0": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 0"),
                        "1": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 1"),
                        "2": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 2"),
                        "3": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 3"),
                        "4": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 4"),
                        "5": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"Message 5")
                    }
                }
            }
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Client says it has up to sequence 2
    let get_response = server
        .post("/blobs/get")
        .json(&json!({
            "topics": {
                "test-topic": {
                    "log-x": 2
                }
            }
        }))
        .await;

    get_response.assert_status_ok();

    let body: GetBlobsResponse = get_response.json();
    let topic_response = &body.blobs_by_topic["test-topic"];

    // Server should return messages 3, 4, 5
    let log_seqs = &topic_response.blobs["log-x"];
    assert_eq!(log_seqs.len(), 3);
    assert!(log_seqs.contains_key(&3));
    assert!(log_seqs.contains_key(&4));
    assert!(log_seqs.contains_key(&5));

    // No missing since server is ahead
    assert!(topic_response.missing.is_empty());
}
