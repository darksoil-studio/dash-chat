use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::holochain_dir;

pub fn build_menu<R: Runtime>(app_handle: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    app_handle.on_menu_event(|app_handle, menu_event| match menu_event.id().as_ref() {
        "open-logs-folder" => {
            let log_folder = app_handle
                .path()
                .app_log_dir()
                .expect("Could not get app log dir");
            if let Err(err) = opener::reveal(log_folder.clone()) {
                log::error!("Failed to open log dir at {log_folder:?}: {err:?}");
            }
        }
        "factory-reset" => {
            let h = app_handle.clone();
            app_handle
                .dialog()
                .message(
                    "Are you sure you want to perform a factory reset? All your data will be lost.",
                )
                .title("Factory Reset")
                .buttons(MessageDialogButtons::OkCancel)
                .show(move |result| match result {
                    true => {
                        if let Err(err) = std::fs::remove_dir_all(holochain_dir()) {
                            log::error!("Failed to perform factory reset: {err:?}");
                        } else {
                            h.restart();
                        }
                    }
                    false => {}
                });
        }
        _ => {}
    });

    Menu::with_items(
        app_handle,
        &[&Submenu::with_items(
            app_handle,
            "File",
            true,
            &[
                &MenuItem::with_id(
                    app_handle,
                    "open-logs-folder",
                    "Open Logs Folder",
                    true,
                    None::<&str>,
                )?,
                &MenuItem::with_id(
                    app_handle,
                    "factory-reset",
                    "Factory Reset",
                    true,
                    None::<&str>,
                )?,
                &PredefinedMenuItem::close_window(app_handle, None)?,
            ],
        )?],
    )
}
