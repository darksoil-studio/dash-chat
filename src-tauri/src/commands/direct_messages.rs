use dashchat_node::{AgentId, Node, Topic, topic::kind::Chat};
use tauri::State;

#[tauri::command]
pub fn direct_message_chat_id(peer: AgentId, node: State<'_, Node>) -> Topic<Chat> {
    Topic::direct_chat([node.agent_id(), peer])
}
