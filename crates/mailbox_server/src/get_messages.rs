use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{AppState, Message, TopicId, MESSAGES_TABLE};

#[derive(Serialize, Deserialize)]
pub struct GetMessagesRequest {
    pub topic_ids: Vec<TopicId>,
}

#[derive(Serialize, Deserialize)]
pub struct GetMessagesResponse {
    #[serde(with = "serde_bytes")]
    pub messages: HashMap<TopicId, Vec<Message>>,
}

mod serde_bytes {
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

pub async fn get_messages_for_topics(
    State(state): State<AppState>,
    Json(payload): Json<GetMessagesRequest>,
) -> Result<Json<GetMessagesResponse>, (StatusCode, String)> {
    let mut messages: HashMap<TopicId, Vec<Message>> = HashMap::new();

    let read_txn = state.db.begin_read().map_err(|e| {
        tracing::error!("Failed to begin read transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to begin transaction: {}", e),
        )
    })?;

    let table = read_txn.open_table(MESSAGES_TABLE).map_err(|e| {
        tracing::error!("Failed to open table: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open table: {}", e),
        )
    })?;

    for topic_id in &payload.topic_ids {
        let mut topic_messages: Vec<Message> = Vec::new();

        let prefix = format!("{}:", topic_id);
        let range_start = prefix.as_str();

        // Create an upper bound by incrementing the last character
        let mut range_end = prefix.clone();
        range_end.push(char::MAX);
        let range_end_str = range_end.as_str();

        for entry in table.range(range_start..range_end_str).map_err(|e| {
            tracing::error!("Failed to create range iterator: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create range iterator: {}", e),
            )
        })? {
            let (_, value) = entry.map_err(|e| {
                tracing::error!("Failed to read entry: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read entry: {}", e),
                )
            })?;

            topic_messages.push(value.value().to_vec());
        }

        messages.insert(topic_id.clone(), topic_messages);
    }

    tracing::debug!("Retrieved messages for {} topics", payload.topic_ids.len());
    Ok(Json(GetMessagesResponse { messages }))
}
