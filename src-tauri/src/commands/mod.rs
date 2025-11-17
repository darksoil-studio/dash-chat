use dashchat_node::Node;
use p2panda_core::PublicKey;
use tauri::{command, State};

pub mod logs;

pub mod contacts;
pub mod profile;

pub mod chats;
pub mod group_chat;

#[command]
pub async fn my_pub_key(node: State<'_, Node>) -> Result<PublicKey, String> {
    Ok(node.public_key().into())
}
