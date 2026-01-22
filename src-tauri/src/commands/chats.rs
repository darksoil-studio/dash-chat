// #[command]
// pub async fn create_group_chat(group_chat_id: GroupChatId, node: State<'_, Node>) -> Result<(), String> {
//     node.create_group_chat_space(group_chat_id)
//         .await
//         .map_err(|e| format!("Failed to create group: {e:?}"))
// }

// #[command]
// pub async fn get_group_chats(node: State<'_, Node>) -> Result<Vec<ChatId>, String> {
//     node.get_groups()
//         .await
//         .map_err(|e| format!("Failed to get groups: {e:?}"))
// }
