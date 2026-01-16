use dashchat_node::{Node, Topic, topic::kind};
use tauri::State;


#[tauri::command]
pub fn my_device_group_topic(node: State<'_, Node>) -> Topic<kind::DeviceGroup> {
    node.device_group_topic()
}
