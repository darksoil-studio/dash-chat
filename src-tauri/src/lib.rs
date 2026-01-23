use dashchat_node::Node;
use mailbox_client::toy::ToyMailboxClient;
use p2panda_core::{cbor::encode_cbor, Body};
use tauri::{Emitter, RunEvent, Manager};

use crate::{
    commands::logs::simplify,
    local_store::{cleanup_local_store_path, local_store_path},
};

mod commands;
mod local_store;
mod utils;

#[cfg(not(mobile))]
mod menu;
#[cfg(mobile)]
mod push_notifications;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    if tauri::is_dev() {
        // MCP for Claude Code to control the tauri app
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }
    #[cfg(mobile)]
    {
        builder = builder
            .plugin(tauri_plugin_barcode_scanner::init())
            .plugin(tauri_plugin_safe_area_insets_css::init());
    }
    #[cfg(not(mobile))]
    {
        builder = builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .menu(|handle| menu::build_menu(handle));
        // app.handle()
        //     .plugin(tauri_plugin_single_instance::init(move |app, argv, cwd| {
        //         // h.emit(
        //         //     "single-instance",
        //         //     Payload { args: argv, cwd },
        //         // )
        //         // .unwrap();
        //     }))?;
    }

    builder
        .invoke_handler(tauri::generate_handler![
            // commands::my_pub_key,
            commands::logs::get_log,
            commands::logs::get_authors,
            commands::profile::set_profile,
            commands::devices::my_device_group_topic,
            commands::contacts::my_agent_id,
            commands::contacts::create_contact_code,
            commands::contacts::add_contact,
            // commands::chats::create_group,
            // commands::group_chat::add_member,
            // commands::group_chat::send_message,
            // commands::group_chat::get_messages,
        ])
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .level_for("dashchat_node", log::LevelFilter::Debug)
                .level_for("tauri_app_lib", log::LevelFilter::Debug) // dash-chat crate
                .build(),
        )
        // .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let handle = app.handle().clone();

            let local_store_path: std::path::PathBuf = local_store_path(&handle)?;
            log::info!("Using local store path: {local_store_path:?}");

            tauri::async_runtime::block_on(async move {
                let local_store = dashchat_node::LocalStore::new(local_store_path).unwrap();
                let config = dashchat_node::NodeConfig::default();
                let (notification_tx, mut notification_rx) = tokio::sync::mpsc::channel(100);
                let node = dashchat_node::Node::new(local_store, config, Some(notification_tx))
                    .await
                    .expect("Failed to create node");

                let mailbox_url = if tauri::is_dev() {
                    "http://localhost:3000"
                } else {
                    "https://mailbox-server.production.dash-chat.dash-chat.garnix.me"
                };

                let mailbox_client = ToyMailboxClient::new(mailbox_url);
                node.mailboxes.add(mailbox_client).await;

                handle.manage(node);

                tauri::async_runtime::spawn(async move {
                    while let Some(notification) = notification_rx.recv().await {
                        log::info!("Received notification: {:?}", notification);

                        let body = match encode_cbor(&notification.payload) {
                            Ok(body) => body,
                            Err(err) => {
                                log::error!("Failed to serialize payload: {err:?}");
                                continue;
                            }
                        };
                        let _node = handle.state::<Node>();
                        let simplified_operation =
                            match simplify(notification.header, Some(Body::new(&body[..]))) {
                                Ok(o) => o,
                                Err(err) => {
                                    log::error!("Failed to simplify operation: {err:?}");
                                    continue;
                                }
                            };

                        if let Err(err) =
                            handle.emit("p2panda://new-operation", simplified_operation)
                        {
                            log::error!("Failed to emit operation: {err:?}");
                        }
                    }
                });
            });

            // app.handle()
            //     .listen("holochain://setup-completed", move |_event| {
            //         let handle2 = handle.clone();
            //         tauri::async_runtime::spawn(async move {
            //             if let Err(err) = setup(handle2.clone()).await {
            //                 log::error!("Failed to setup: {err:?}");
            //                 return;
            //             }

            //             #[cfg(mobile)]
            //             if let Err(err) =
            //                 push_notifications::setup_push_notifications(handle2.clone())
            //             {
            //                 log::error!("Failed to setup push notifications: {err:?}");
            //             }
            //         });
            //         let handle = handle.clone();
            //         tauri::async_runtime::spawn(async move {
            //             if let Err(err) = open_window(handle.clone()).await {
            //                 log::error!("Failed to setup: {err:?}");
            //             }
            //         });
            //     });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| match event {
            // Limitation: this won't fire when running pnpm start with mprocs,
            // only when the tauri app is closed directly
            RunEvent::Exit => cleanup_local_store_path(app_handle).expect("Failed to cleanup"),
            _ => {}
        });

    ()
}
