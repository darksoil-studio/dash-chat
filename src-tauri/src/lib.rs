use holochain_types::app::AppBundle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
#[cfg(not(mobile))]
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri_plugin_holochain::{vec_to_locked, HolochainExt, HolochainPluginConfig, WANNetworkConfig};

const APP_ID: &'static str = "messenger-demo";
const SIGNAL_URL: &'static str = "wss://sbd.holo.host";
const BOOTSTRAP_URL: &'static str = "https://bootstrap.holo.host";

pub fn happ_bundle() -> AppBundle {
    let bytes = include_bytes!("../../workdir/messenger-demo.happ");
    AppBundle::decode(bytes).expect("Failed to decode messenger-demo happ")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::env::set_var("WASM_LOG", "info");
    let mut config = HolochainPluginConfig::new(holochain_dir(), wan_network_config());
    // config.gossip_arc_clamp = None;

    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_holochain::init(
            vec_to_locked(vec![]).expect("Can't build passphrase"),
            config
        ))
        .setup(|app| {
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            #[cfg(not(mobile))]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            #[cfg(not(mobile))]
            app.handle()
                .on_menu_event(|app_handle, menu_event| match menu_event.id().as_ref() {
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
                                .message("Are you sure you want to perform a factory reset? All your data will be lost.")
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
                                    false => {
            
                                    }
                                });
                    }
                    _ => {}
                });

            let handle = app.handle().clone();
            let result: anyhow::Result<()> = tauri::async_runtime::block_on(async move {
                setup(handle).await?;

                // After set up we can be sure our app is installed and up to date, so we can just open it
                let mut window_builder = app
                    .holochain()?
                    .main_window_builder(
                        String::from(APP_ID),
                        true,
                        Some(String::from(APP_ID)),
                        None,
                    )
                    .await?;

                #[cfg(desktop)]
                {
                    window_builder = window_builder
                        .title(String::from("Messenger Demo"))
                        .inner_size(1400.0, 1000.0);
                }

                window_builder.build()?;

                Ok(())
            });

            result?;

            Ok(())
        });

    #[cfg(not(mobile))]
    {
        builder = builder.menu(|handle| {
            Menu::with_items(
                handle,
                &[&Submenu::with_items(
                    handle,
                    "File",
                    true,
                    &[
                        &MenuItem::with_id(
                            handle,
                            "open-logs-folder",
                            "Open Logs Folder",
                            true,
                            None::<&str>,
                        )?,
                        &MenuItem::with_id(
                            handle,
                            "factory-reset",
                            "Factory Reset",
                            true,
                            None::<&str>,
                        )?,
                        &PredefinedMenuItem::close_window(handle, None)?,
                    ],
                )?],
            )
        });
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Very simple setup for now:
// - On app start, check whether the app is already installed:
//   - If it's not installed, install it
//   - If it's installed, check if it's necessary to update the coordinators for our hApp,
//     and do so if it is
//
// You can modify this function to suit your needs if they become more complex
async fn setup(handle: AppHandle) -> anyhow::Result<()> {
    let admin_ws = handle.holochain()?.admin_websocket().await?;

    let installed_apps = admin_ws
        .list_apps(None)
        .await
        .map_err(|err| tauri_plugin_holochain::Error::ConductorApiError(err))?;

    if installed_apps
        .iter()
        .find(|app| app.installed_app_id.as_str().eq(APP_ID))
        .is_none()
    {
        handle
            .holochain()?
            .install_app(String::from(APP_ID), happ_bundle(), None, None, None)
            .await?;

        Ok(())
    } else {
        handle
            .holochain()?
            .update_app_if_necessary(String::from(APP_ID), happ_bundle())
            .await?;

        Ok(())
    }
}

fn wan_network_config() -> Option<WANNetworkConfig> {
    // if tauri::is_dev() {
    //     None
    // } else {
        Some(WANNetworkConfig {
            signal_url: url2::url2!("{}", SIGNAL_URL),
            bootstrap_url: url2::url2!("{}", BOOTSTRAP_URL),
            ice_servers_urls: vec![],
        })
    // }
}

fn holochain_dir() -> PathBuf {
    if tauri::is_dev() {
        #[cfg(target_os = "android")]
        {
            app_dirs2::app_root(
                app_dirs2::AppDataType::UserCache,
                &app_dirs2::AppInfo {
                    name: "messenger-demo",
                    author: std::env!("CARGO_PKG_AUTHORS"),
                },
            )
            .expect("Could not get the UserCache directory")
        }
        #[cfg(not(target_os = "android"))]
        {
            let tmp_dir = tempdir::TempDir::new("messenger-demo")
                .expect("Could not create temporary directory");

            // Convert `tmp_dir` into a `Path`, destroying the `TempDir`
            // without deleting the directory.
            let tmp_path = tmp_dir.into_path();
            tmp_path
        }
    } else {
        app_dirs2::app_root(
            app_dirs2::AppDataType::UserData,
            &app_dirs2::AppInfo {
                name: "messenger-demo",
                author: std::env!("CARGO_PKG_AUTHORS"),
            },
        )
        .expect("Could not get app root")
        .join("holochain")
    }
}
