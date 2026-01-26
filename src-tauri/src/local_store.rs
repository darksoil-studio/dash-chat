use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// In production, use the local data dir from the operating system
// In development, use a temporary directory
pub fn local_store_path(handle: &AppHandle) -> anyhow::Result<PathBuf> {
    if tauri::is_dev() {
        let dir = tempfile::tempdir()?;
        let dir_path = dir.path();
        let path = dir_path.join("localdata.redb");
        handle.manage(dir);
        Ok(path)
    } else {
        Ok(handle.path().local_data_dir()?.join("localdata.redb"))
    }
}

pub fn cleanup_local_store_path(app_handle: &AppHandle) -> anyhow::Result<()> {
    if tauri::is_dev() {
        use tempfile::TempDir;

        let path = app_handle.state::<TempDir>();
        std::fs::remove_dir_all(path.inner())?;
    }
    Ok(())
}
