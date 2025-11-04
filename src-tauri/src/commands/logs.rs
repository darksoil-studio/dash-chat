use dashchat_node::{Header, Node, Payload, Topic, PK};
use p2panda_core::{Body, PublicKey};
use tauri::State;

#[tauri::command]
pub async fn get_log(
    topic_id: Topic,
    author: PK,
    node: State<'_, Node>,
) -> Result<Vec<(Header, Option<Body>)>, String> {
    let log = node
        .get_log(topic_id, author)
        .await
        .map_err(|e| format!("Failed to get log: {e:?}"))?;
    Ok(log)
}
