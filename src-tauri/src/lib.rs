use anyhow::anyhow;
use std::path::PathBuf;
use tauri::{AppHandle, Listener, WebviewWindow};
use tauri_plugin_holochain::{
    vec_to_locked, AppBundle, AppStatusFilter, DnaModifiersOpt, HolochainExt,
    HolochainPluginConfig, NetworkConfig, RoleSettings, RoleSettingsMap,
};

mod utils;
pub use utils::migrate_app;

#[cfg(not(mobile))]
mod menu;
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
    std::env::set_var("WASM_LOG", "debug");

    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .level_for("tracing::span", log::LevelFilter::Off)
                .level_for("iroh", log::LevelFilter::Warn)
                .level_for("holochain", log::LevelFilter::Warn)
                .level_for("kitsune2", log::LevelFilter::Warn)
                .level_for("kitsune2_gossip", log::LevelFilter::Warn)
                .level_for("kitsune2_transport_iroh", log::LevelFilter::Debug)
                .level_for("holochain_runtime", log::LevelFilter::Info)
                .level_for("dash-chat", log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_holochain::async_init(
            vec_to_locked(vec![]),
            HolochainPluginConfig::new(holochain_dir(), network_config()).enable_mdns_discovery(),
        ))
        .setup(move |app| {
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            #[cfg(not(mobile))]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            let handle = app.handle().clone();

            app.handle()
                .listen("holochain://setup-completed", move |_event| {
                    let handle2 = handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(err) = setup(handle2.clone()).await {
                            log::error!("Failed to setup: {err:?}");
                            return;
                        }

                        #[cfg(mobile)]
                        if let Err(err) =
                            push_notifications::setup_push_notifications(handle2.clone())
                        {
                            log::error!("Failed to setup push notifications: {err:?}");
                        }
                    });
                    let handle = handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(err) = open_window(handle.clone()).await {
                            log::error!("Failed to setup: {err:?}");
                        }
                    });
                });

            Ok(())
        });

    #[cfg(not(mobile))]
    {
        builder = builder.menu(|handle| menu::build_menu(handle));
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn open_window(handle: AppHandle) -> anyhow::Result<WebviewWindow> {
    let mut window_builder = handle
        .holochain()?
        .main_window_builder(String::from("main"), true, Some(app_id()), None)
        .await?;

    #[cfg(not(mobile))]
    {
        window_builder = window_builder
            .title(String::from("Dash Chat"))
            .inner_size(1400.0, 1000.0);
    }

    let window = window_builder.build()?;
    Ok(window)
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
            .filter(|app| app.installed_app_id.as_str().starts_with(APP_ID_PREFIX))
            .min_by_key(|app_info| app_info.installed_at);

        let services_network_seed = String::from("somesecretnetworkseed");

        let mut roles_settings: RoleSettingsMap = RoleSettingsMap::new();
        roles_settings.insert(
            String::from("services"),
            RoleSettings::Provisioned {
                membrane_proof: None,
                modifiers: Some(DnaModifiersOpt {
                    network_seed: Some(services_network_seed),
                    ..Default::default()
                }),
            },
        );

        if let Some(previous_app) = previous_app {
            migrate_app(
                &handle.holochain()?.holochain_runtime,
                previous_app.installed_app_id.clone(),
                app_id(),
                happ_bundle(),
                Some(roles_settings),
            )
            .await?;

            admin_ws
                .disable_app(previous_app.installed_app_id.clone())
                .await
                .map_err(|err| anyhow!("{err:?}"))?;
        } else {
            handle
                .holochain()?
                .install_app(
                    String::from(app_id()),
                    happ_bundle(),
                    Some(roles_settings),
                    None,
                    None,
                )
                .await?;
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
    } else {
        network_config.bootstrap_url =
            url2::Url2::parse("https://bootstrap.kitsune-v0-1.kitsune.darksoil-studio.garnix.me");
    }

    // Don't hold any slice of the DHT in mobile
    if cfg!(mobile) {
        network_config.target_arc_factor = 0;
    }

    network_config
}

fn holochain_dir() -> PathBuf {
    if tauri::is_dev() && cfg!(not(mobile)) {
        let tmp_dir =
            tempdir::TempDir::new("dash-chat").expect("Could not create temporary directory");

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
