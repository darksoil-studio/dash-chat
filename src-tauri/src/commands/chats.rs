use dashchat_node::{ChatId, Node, PK};
use tauri::{command, State};

#[command]
pub async fn create_group(chat_id: ChatId, node: State<'_, Node>) -> Result<(), String> {
    node.create_group(chat_id)
        .await
        .map_err(|e| format!("Failed to create group: {e:?}"))
}

#[command]
pub async fn get_groups(node: State<'_, Node>) -> Result<Vec<ChatId>, String> {
    node.get_groups()
        .await
        .map_err(|e| format!("Failed to get groups: {e:?}"))
}
