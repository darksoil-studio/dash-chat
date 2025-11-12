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
pub async fn add_contact(
    member_code: MemberCode,
    node: State<'_, Node>,
) -> Result<PublicKey, String> {
    let pk = node
        .add_contact(member_code)
        .await
        .map_err(|e| format!("Failed to add contact: {e:?}"))?;

    Ok(pk.into())
}

// #[tauri::command]
// pub async fn remove_contact(
//     contact_id: PublicKey,
//     node: State<'_, Node>,
// ) -> Result<PublicKey, String> {
//     node.remove_contact(contact_id.into())
//         .await
//         .map_err(|e| format!("Failed to remove contact: {e:?}"))
// }

#[tauri::command]
pub async fn get_contacts(node: State<'_, Node>) -> Result<Vec<PublicKey>, String> {
    let pks = node
        .get_contacts()
        .await
        .map_err(|e| format!("Failed to get my contacts: {e:?}"))?;

    let pks = pks.into_iter().map(|pk| pk.into()).collect();

    Ok(pks)
}
