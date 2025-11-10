use dashchat_node::{Node, Profile};
use tauri::State;

#[tauri::command]
pub async fn set_profile(profile: Profile, node: State<'_, Node>) -> Result<(), String> {
    node.set_profile(profile)
        .await
        .map_err(|e| format!("Failed to get log: {e:?}"))?;
    Ok(())
}
