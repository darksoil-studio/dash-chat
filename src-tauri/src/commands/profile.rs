use dashchat_node::{DashChatTopicId, Header, Node, Payload, Profile, Topic, PK};
use p2panda_core::{Body, PublicKey};
use p2panda_net::TopicId;
use tauri::State;

#[tauri::command]
pub async fn set_profile(profile: Profile, node: State<'_, Node>) -> Result<(), String> {
    node.set_profile(profile)
        .await
        .map_err(|e| format!("Failed to get log: {e:?}"))?;
    Ok(())
}
