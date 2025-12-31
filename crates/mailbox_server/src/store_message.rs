use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, Message, TopicId, MESSAGES_TABLE};

#[derive(Serialize, Deserialize)]
pub struct StoreMessageRequest {
    pub topic_id: TopicId,
    #[serde(with = "serde_bytes")]
    pub message: Message,
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

pub async fn store_message(
    State(state): State<AppState>,
    Json(payload): Json<StoreMessageRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let key = format!("{}:{}", payload.topic_id, uuid::Uuid::now_v7());

    let write_txn = state.db.begin_write().map_err(|e| {
        tracing::error!("Failed to begin write transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to begin transaction: {}", e),
        )
    })?;

    {
        let mut table = write_txn.open_table(MESSAGES_TABLE).map_err(|e| {
            tracing::error!("Failed to open table: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open table: {}", e),
            )
        })?;

        table.insert(key.as_str(), payload.message.as_slice()).map_err(|e| {
            tracing::error!("Failed to insert message: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to insert message: {}", e),
            )
        })?;
    }

    write_txn.commit().map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to commit transaction: {}", e),
        )
    })?;

    tracing::debug!("Stored message for topic_id: {}", payload.topic_id);
    Ok(StatusCode::CREATED)
}
