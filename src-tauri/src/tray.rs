use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{
    menu::{Menu, MenuItem},
    App, AppHandle, Manager, Runtime,
};

pub fn build_tray<R: Runtime>(app: &App<R>) -> tauri::Result<TrayIcon<R>> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    // TODO: add separator
    let title = MenuItem::new(app, "DashChat Local Mailbox", false, None::<&str>)?;
    let menu = Menu::with_items(app, &[&title, &quit_i])?;

    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, menu_event| match menu_event.id().as_ref() {
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    Ok(tray)
}

pub fn toggle_tray<R: Runtime>(
    app_handle: &AppHandle<R>,
    enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let tray = app_handle.state::<TrayIcon<R>>();

    tray.set_visible(enabled)?;

    Ok(())
}
