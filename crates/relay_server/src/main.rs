use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Router,
    Json,
};
use scylla::{Session, SessionBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type TopicId = String;
type Message = Vec<u8>;

#[derive(Clone)]
struct AppState {
    db: Arc<Session>,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize, Deserialize)]
struct StoreMessageRequest {
    topic_id: TopicId,
    #[serde(with = "serde_bytes")]
    message: Message,
}

#[derive(Serialize, Deserialize)]
struct GetMessagesRequest {
    topic_ids: Vec<TopicId>,
}

#[derive(Serialize, Deserialize)]
struct GetMessagesResponse {
    #[serde(with = "serde_bytes_map")]
    messages: HashMap<TopicId, Vec<Message>>,
}

mod serde_bytes {
    use serde::{Deserialize, Deserializer, Serializer};
    use base64::{Engine as _, engine::general_purpose};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&general_purpose::STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        general_purpose::STANDARD.decode(s).map_err(serde::de::Error::custom)
    }
}

mod serde_bytes_map {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::collections::HashMap;
    use base64::{Engine as _, engine::general_purpose};

    pub fn serialize<S>(map: &HashMap<String, Vec<Vec<u8>>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut ser_map = serializer.serialize_map(Some(map.len()))?;
        for (k, messages) in map {
            let encoded_messages: Vec<String> = messages
                .iter()
                .map(|msg| general_purpose::STANDARD.encode(msg))
                .collect();
            ser_map.serialize_entry(k, &encoded_messages)?;
        }
        ser_map.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Vec<Vec<u8>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, Vec<String>> = HashMap::deserialize(deserializer)?;
        let mut result = HashMap::new();
        for (k, encoded_messages) in map {
            let messages: Result<Vec<Vec<u8>>, _> = encoded_messages
                .iter()
                .map(|s| general_purpose::STANDARD.decode(s))
                .collect();
            result.insert(k, messages.map_err(serde::de::Error::custom)?);
        }
        Ok(result)
    }
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn store_message(
    State(state): State<AppState>,
    Json(payload): Json<StoreMessageRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let message_id = scylla::frame::value::CqlTimeuuid::from(uuid::Uuid::now_v1(&[0; 6]));

    state
        .db
        .query_unpaged(
            "INSERT INTO messages (topic_id, message_id, message) VALUES (?, ?, ?)",
            (&payload.topic_id, message_id, &payload.message),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to store message: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to store message: {}", e),
            )
        })?;

    tracing::debug!("Stored message for topic_id: {}", payload.topic_id);
    Ok(StatusCode::CREATED)
}

async fn get_messages_for_topics(
    State(state): State<AppState>,
    Json(payload): Json<GetMessagesRequest>,
) -> Result<Json<GetMessagesResponse>, (StatusCode, String)> {
    let mut messages: HashMap<TopicId, Vec<Message>> = HashMap::new();

    for topic_id in &payload.topic_ids {
        let query_result = state
            .db
            .query_unpaged("SELECT message FROM messages WHERE topic_id = ?", (topic_id,))
            .await
            .map_err(|e| {
                tracing::error!("Failed to query messages for topic {}: {}", topic_id, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to query messages: {}", e),
                )
            })?;

        let rows = query_result.rows().map_err(|e| {
            tracing::error!("Failed to get rows: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get rows: {}", e),
            )
        })?;

        let topic_messages: Vec<Message> = rows
            .into_iter()
            .filter_map(|row| {
                row.columns.first()
                    .and_then(|col| col.as_ref())
                    .and_then(|val| {
                        if let scylla::frame::response::result::CqlValue::Blob(bytes) = val {
                            Some(bytes.clone())
                        } else {
                            None
                        }
                    })
            })
            .collect();

        messages.insert(topic_id.clone(), topic_messages);
    }

    tracing::debug!("Retrieved messages for {} topics", payload.topic_ids.len());
    Ok(Json(GetMessagesResponse { messages }))
}

async fn init_db() -> Result<Session, Box<dyn std::error::Error>> {
    let uri = std::env::var("CASSANDRA_URI").unwrap_or_else(|_| "10.0.0.9:9042".to_string());
    let keyspace = std::env::var("CASSANDRA_KEYSPACE").unwrap_or_else(|_| "relay".to_string());

    tracing::info!("Connecting to Cassandra at {}", uri);

    let session = SessionBuilder::new()
        .known_node(uri)
        .build()
        .await?;

    tracing::info!("Successfully connected to Cassandra");

    session
        .query_unpaged(
            format!(
                "CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{'class': 'SimpleStrategy', 'replication_factor': 1}}",
                keyspace
            ),
            &[],
        )
        .await?;

    session.use_keyspace(&keyspace, false).await?;

    session
        .query_unpaged(
            "CREATE TABLE IF NOT EXISTS messages (
                topic_id text,
                message_id timeuuid,
                message blob,
                PRIMARY KEY (topic_id, message_id)
            ) WITH CLUSTERING ORDER BY (message_id DESC)",
            &[],
        )
        .await?;

    tracing::info!("Database schema initialized");

    Ok(session)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "relay_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = init_db().await?;

    let state = AppState {
        db: Arc::new(db),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/messages/store", post(store_message))
        .route("/messages/get", post(get_messages_for_topics))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let addr = listener.local_addr()?;

    tracing::info!("Relay server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
