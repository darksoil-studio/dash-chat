use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// In production, use the local data dir from the operating system
// In development, use a numbered directory in the local data dir
pub fn local_data_dir(handle: &AppHandle) -> anyhow::Result<PathBuf> {
    let local_data_path = if tauri::is_dev() {
        let base = match std::env::var("DEV_DBS_PATH") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                let mut p = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
                p.pop();
                p.join(".dev-dbs")
            }
        };
        base.join(format!("agent-{}", std::env::var("AGENT")?))
    } else {
        handle.path().local_data_dir()?
    };
    if !local_data_path.exists() {
        std::fs::create_dir_all(&local_data_path)?;
    }
    Ok(local_data_path)
}
