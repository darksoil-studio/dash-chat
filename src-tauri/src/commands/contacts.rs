use dashchat_node::{
    AgentId, topic::DashChatTopicId, Header, Node, Payload, Profile, QrCode, ShareIntent,
};
use p2panda_core::{Body, PublicKey};
use p2panda_net::TopicId;
use tauri::State;

#[tauri::command]
pub async fn create_contact_code(node: State<'_, Node>) -> Result<QrCode, String> {
    node.new_qr_code(ShareIntent::AddContact, false)
        .await
        .map_err(|e| format!("Failed to create contact code: {e:?}"))
}

#[tauri::command]
pub fn my_agent_id(node: State<'_, Node>) -> AgentId {
    node.agent_id()
}

#[tauri::command]
pub async fn add_contact(contact_code: QrCode, node: State<'_, Node>) -> Result<(), String> {
    node.add_contact(contact_code.clone())
        .await
        .map_err(|e| format!("Failed to add contact: {e:?}"))?;

    Ok(())
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

// #[tauri::command]
// pub async fn get_contacts(node: State<'_, Node>) -> Result<Vec<PublicKey>, String> {
//     let pks = node
//         .get_contacts()
//         .await
//         .map_err(|e| format!("Failed to get my contacts: {e:?}"))?;

//     let pks = pks.into_iter().map(|pk| pk.into()).collect();

//     Ok(pks)
// }
