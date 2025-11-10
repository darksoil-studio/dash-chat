use dashchat_node::{DashChatTopicId, Header, Node};
use p2panda_core::{cbor::decode_cbor, Body, Hash, PublicKey};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SimplifiedOperation {
    // hash: Hash,
    pub header: SimplifiedHeader,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SimplifiedHeader {
    /// Author of this operation.
    public_key: PublicKey,

    /// Time in microseconds since the Unix epoch.
    timestamp: u64,

    /// Number of operations this author has published to this log, begins with 0 and is always
    /// incremented by 1 with each new operation by the same author.
    seq_num: u64,

    /// Hash of the previous operation of the same author and log. Can be omitted if first
    /// operation in log.
    backlink: Option<Hash>,

    /// List of hashes of the operations we refer to as the "previous" ones. These are operations
    /// from other authors. Can be left empty if no partial ordering is required or no other
    /// author has been observed yet.
    previous: Vec<Hash>,

    topic_id: DashChatTopicId,
}

impl From<Header> for SimplifiedHeader {
    fn from(header: Header) -> SimplifiedHeader {
        SimplifiedHeader {
            public_key: header.public_key,
            timestamp: header.timestamp,
            seq_num: header.seq_num,
            backlink: header.backlink,
            previous: header.previous,
            topic_id: header.extensions.log_id,
        }
    }
}

pub fn simplify(
    // hash: Hash,
    header: Header,
    body: Option<Body>,
) -> Result<SimplifiedOperation, String> {
    let body: Option<serde_json::Value> = match body {
        Some(b) => decode_cbor(&b.to_bytes()[..]).map_err(|err| format!("{err:?}"))?,
        None => None,
    };

    let operation = SimplifiedOperation {
        // hash,
        header: SimplifiedHeader::from(header),
        body,
    };

    Ok(operation)
}

#[tauri::command]
pub async fn get_log(
    topic_id: DashChatTopicId,
    author: PublicKey,
    node: State<'_, Node>,
) -> Result<Vec<SimplifiedOperation>, String> {
    let log = node
        .get_log(topic_id, author)
        .await
        .map_err(|e| format!("Failed to get log: {e:?}"))?;

    let simplified_log = log
        .into_iter()
        .map(|(header, body)| simplify(header, body))
        .collect::<Result<Vec<SimplifiedOperation>, String>>()?;

    Ok(simplified_log)
}

#[tauri::command]
pub async fn get_authors(
    topic_id: DashChatTopicId,
    node: State<'_, Node>,
) -> Result<std::collections::HashSet<PublicKey>, String> {
    let authors = node
        .get_authors(topic_id)
        .await
        .map_err(|e| format!("Failed to get log: {e:?}"))?;
    Ok(authors)
}
