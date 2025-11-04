use dashchat_node::Node;
use tauri::{command, State};

pub mod logs;
pub mod request;

#[command]
pub async fn my_pub_key(node: State<'_, Node>) -> Result<String, String> {
    Ok(node.public_key().to_string())
}
