use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// In production, use the local data dir from the operating system
// In development, use a temporary directory
pub fn local_store_path(handle: &AppHandle) -> anyhow::Result<PathBuf> {
    if cfg!(debug_assertions) {
        let dir = tempfile::tempdir()?;
        let path = dir.keep().join("localdata.db");
        Ok(path)
    } else {
        Ok(handle.path().local_data_dir()?.join("localdata.redb"))
    }
}
