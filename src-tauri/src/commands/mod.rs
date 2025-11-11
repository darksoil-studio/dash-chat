use dashchat_node::{Node, PK};
use p2panda_core::PublicKey;
use tauri::{command, State};

pub mod logs;
pub mod profile;
pub mod friends;

#[command]
pub async fn my_pub_key(node: State<'_, Node>) -> Result<PublicKey, String> {
    Ok(node.public_key())
}
