use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// In production, use the local data dir from the operating system
// In development, use DEV_DBS_PATH/agent-{AGENT} (set in mprocs.yaml)
pub fn local_data_dir(handle: &AppHandle) -> anyhow::Result<PathBuf> {
    let local_data_path = if tauri::is_dev() {
        let base = PathBuf::from(std::env::var("DEV_DBS_PATH")?);
        base.join(format!("agent-{}", std::env::var("AGENT")?))
    } else {
        handle.path().local_data_dir()?
    };
    if !local_data_path.exists() {
        std::fs::create_dir_all(&local_data_path)?;
    }
    Ok(local_data_path)
}
