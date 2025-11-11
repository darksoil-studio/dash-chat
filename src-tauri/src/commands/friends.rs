use dashchat_node::{DashChatTopicId, Header, MemberCode, Node, Payload, Profile, Topic, PK};
use p2panda_core::{Body, PublicKey};
use p2panda_net::TopicId;
use tauri::State;

#[tauri::command]
pub async fn my_member_code(node: State<'_, Node>) -> Result<MemberCode, String> {
    node.me()
        .await
        .map_err(|e| format!("Failed to get my member code: {e:?}"))
}

#[tauri::command]
pub async fn add_friend(
    member_code: MemberCode,
    node: State<'_, Node>,
) -> Result<PublicKey, String> {
    let pk = node
        .add_friend(member_code)
        .await
        .map_err(|e| format!("Failed to add friend: {e:?}"))?;

    Ok(pk.into())
}

// #[tauri::command]
// pub async fn remove_friend(
//     friend_id: PublicKey,
//     node: State<'_, Node>,
// ) -> Result<PublicKey, String> {
//     node.remove_friend(friend_id.into())
//         .await
//         .map_err(|e| format!("Failed to remove friend: {e:?}"))
// }
