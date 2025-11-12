use dashchat_node::{ChatId, ChatMessageContent, Node};
use p2panda_core::PublicKey;
use tauri::{command, State};

#[command]
pub async fn add_member(
    chat_id: ChatId,
    member: PublicKey,
    node: State<'_, Node>,
) -> Result<(), String> {
    node.add_member(chat_id, member.into())
        .await
        .map_err(|e| format!("Failed to add member: {e:?}"))
}

#[command]
pub async fn send_message(
    chat_id: ChatId,
    content: ChatMessageContent,
    node: State<'_, Node>,
) -> Result<(), String> {
    node.send_message(chat_id, content)
        .await
        .map_err(|e| format!("Failed to send message: {e:?}"))?;

    Ok(())
}
