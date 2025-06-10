use anyhow::anyhow;
use holochain_client::{AppStatusFilter, CellInfo, ExternIO, ZomeCallTarget};
use holochain_types::app::AppBundle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use std::path::PathBuf;
use tauri::{AppHandle, Listener, Manager};
#[cfg(not(mobile))]
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri_plugin_holochain::{vec_to_locked, DnaModifiersOpt, HolochainExt, HolochainPluginConfig, NetworkConfig, RoleSettings, RoleSettingsMap};

#[cfg(mobile)]
mod modify_notification;
#[cfg(mobile)]
mod push_notifications;

pub fn happ_bundle() -> AppBundle {
    let bytes = include_bytes!("../../workdir/dash-chat.happ");
    AppBundle::decode(bytes).expect("Failed to decode dash-chat happ")
}

const APP_ID_PREFIX: &'static str = "dash-chat";
const DNA_HASH: &'static str = include_str!("../../workdir/dash-chat-dna_hashes");

fn app_id() -> String {
    format!("{APP_ID_PREFIX}-{}", DNA_HASH.trim())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::env::set_var("WASM_LOG", "info");
    let config = HolochainPluginConfig::new(holochain_dir(), network_config());

    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_holochain::async_init(
            vec_to_locked(vec![]),
            config
        ))
        .setup(move |app| {

            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            #[cfg(not(mobile))]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
                
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
            }
            let handle = app.handle().clone();

            app.handle().listen("holochain://setup-completed", move |_event| {
                let handle2 = handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = setup(handle2.clone()).await {
                        log::error!("Failed to setup: {err:?}");
                        return;
                    }

                    #[cfg(mobile)]
                    if let Err(err) = push_notifications::setup_push_notifications(handle2.clone()) {
                        log::error!("Failed to setup push notifications: {err:?}");
                    }
                });
                let handle = handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = open_window(handle.clone()).await{
                        log::error!("Failed to setup: {err:?}");
                    }
                });

            });

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

async fn open_window(handle: AppHandle) -> anyhow::Result<() > {
    let mut window_builder = handle
        .holochain()?
        .main_window_builder(
            String::from("main"),
            true,
            Some(app_id()),
            None,
        )
        .await?;

    #[cfg(not(mobile))]
    {
        window_builder = window_builder
            .title(String::from("Dash Chat"))
            .inner_size(1400.0, 1000.0);
    }

    window_builder.build()?;
    Ok(())
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
        .list_apps(Some(AppStatusFilter::Running))
        .await
        .map_err(|err| tauri_plugin_holochain::Error::ConductorApiError(err))?;

    let app_is_already_installed = installed_apps
        .iter()
        .find(|app| app.installed_app_id.as_str().eq(&app_id()))
        .is_some();

    if !app_is_already_installed {
        let previous_app = installed_apps
            .iter()
            .find(|app| app.installed_app_id.as_str().starts_with(APP_ID_PREFIX));

        let agent_key = previous_app.map(|app| app.agent_pub_key.clone());

        let service_providers_network_seed = String::from("somesecretnetworkseed");

        let mut roles_settings: RoleSettingsMap = RoleSettingsMap::new();
        roles_settings.insert(
            String::from("service_providers"),
            RoleSettings::Provisioned { membrane_proof: None, modifiers: Some(DnaModifiersOpt {
                network_seed: Some(service_providers_network_seed),
                ..Default::default()
            })
        });
        
        handle
            .holochain()?
            .install_app(
                String::from(app_id()),
                happ_bundle(),
                Some(roles_settings),
                agent_key,
                None,
            )
            .await?;

        if let Some(previous_app) = previous_app {
            log::warn!("Migrating from old app {}", previous_app.installed_app_id);
            let Some(Some(CellInfo::Provisioned(previous_cell_info))) = previous_app
                .cell_info
                .get("main")
                .map(|c| c.first())
            else {
                log::error!(
                    "'main' cell was not found in previous app {}",
                    previous_app.installed_app_id
                );
                return Ok(());
            };

            let previous_cell_id = previous_cell_info.cell_id.clone();

            let app_ws = handle.holochain()?.app_websocket(app_id()).await?;
            let migration_result = app_ws
                .call_zome(
                    ZomeCallTarget::RoleName("main".into()),
                    "messenger".into(),
                    "migrate_from_old_cell".into(),
                    ExternIO::encode(previous_cell_id.clone())?,
                )
                .await;

            if let Err(err) = migration_result {
                log::error!("Error migrating data from the previous version of the app: {err:?}",);
                return Ok(());
            }

            let migration_result = app_ws
                .call_zome(
                    ZomeCallTarget::RoleName("main".into()),
                    "friends".into(),
                    "migrate_from_old_cell".into(),
                    ExternIO::encode(previous_cell_id)?,
                )
                .await;

            if let Err(err) = migration_result {
                log::error!("Error migrating data from the previous version of the app: {err:?}",);
                return Ok(());
            }

            admin_ws
                .disable_app(previous_app.installed_app_id.clone())
                .await
                .map_err(|err| anyhow!("{err:?}"))?;
        }

        Ok(())
    } else {
        handle
            .holochain()?
            .update_app_if_necessary(String::from(app_id()), happ_bundle())
            .await?;

        Ok(())
    }
}

fn network_config() -> NetworkConfig {
    let mut network_config = NetworkConfig::default();

    // Don't use the bootstrap service on tauri dev mode
    if tauri::is_dev() {
        network_config.bootstrap_url = url2::Url2::parse("http://0.0.0.0:8888");
        network_config.signal_url = url2::Url2::parse("ws://0.0.0.0:8888");
    } else {
        network_config.bootstrap_url = url2::Url2::parse("http://157.180.93.55:8888");
        network_config.signal_url = url2::Url2::parse("ws://157.180.93.55:8888");
    }

    // Don't hold any slice of the DHT in mobile
    if cfg!(mobile) {
        network_config.target_arc_factor = 0;
    }

    network_config
}

fn holochain_dir() -> PathBuf {
    if tauri::is_dev() {
        let tmp_dir = tempdir::TempDir::new("dash-chat")
            .expect("Could not create temporary directory");

        // Convert `tmp_dir` into a `Path`, destroying the `TempDir`
        // without deleting the directory.
        let tmp_path = tmp_dir.into_path();
        tmp_path
    } else {
        app_dirs2::app_root(
            app_dirs2::AppDataType::UserData,
            &app_dirs2::AppInfo {
                name: "dash-chat",
                author: std::env!("CARGO_PKG_AUTHORS"),
            },
        )
        .expect("Could not get app root")
        .join(get_version())
        .join("holochain")
    }
}

fn get_version() -> String {
    let semver = std::env!("CARGO_PKG_VERSION");

    if semver.starts_with("0.0.") {
        return semver.to_string();
    }

    if semver.starts_with("0.") {
        let v: Vec<&str> = semver.split(".").collect();
        return format!("{}.{}", v[0], v[1]);
    }
    let v: Vec<&str> = semver.split(".").collect();
    return format!("{}", v[0]);
}
