use dashchat_node::Node;
use p2panda_core::{cbor::encode_cbor, Body};
use tauri::{Emitter, Manager};

use crate::commands::logs::simplify;

mod commands;
mod utils;

#[cfg(not(mobile))]
mod menu;
#[cfg(mobile)]
mod push_notifications;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // commands::my_pub_key,
            commands::logs::get_log,
            commands::logs::get_authors,
            commands::profile::set_profile,
            commands::devices::my_device_group_topic,
            commands::contacts::my_chat_actor_id,
            commands::contacts::create_contact_code,
            commands::contacts::add_contact,
            commands::chats::get_group_chats,
            commands::chats::create_group_chat,
            commands::group_chat::add_member,
            commands::group_chat::send_message,
            commands::group_chat::get_messages,
        ])
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .level_for("dash-chat", log::LevelFilter::Debug)
                .build(),
        )
        // .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder = builder
        .setup(move |app| {
            #[cfg(mobile)]
            {
                app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
                app.handle()
                    .plugin(tauri_plugin_safe_area_insets_css::init())?;
            }
            #[cfg(not(mobile))]
            {
                let _h = app.handle();
                // app.handle()
                //     .plugin(tauri_plugin_single_instance::init(move |app, argv, cwd| {
                //         // h.emit(
                //         //     "single-instance",
                //         //     Payload { args: argv, cwd },
                //         // )
                //         // .unwrap();
                //     }))?;

                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            let handle = app.handle().clone();

            tauri::async_runtime::block_on(async move {
                let local_data = dashchat_node::NodeLocalData::new_random();
                let config = dashchat_node::NodeConfig::default();
                let (notification_tx, mut notification_rx) = tokio::sync::mpsc::channel(100);
                let node = dashchat_node::Node::new(local_data, config, Some(notification_tx))
                    .await
                    .expect("Failed to create node");

                handle.manage(node);

                tauri::async_runtime::spawn(async move {
                    while let Some(notification) = notification_rx.recv().await {
                        // TODO: trigger new operation handler
                        println!("Received notification: {:?}", notification);

                        let body = match encode_cbor(&notification.payload) {
                            Ok(body) => body,
                            Err(err) => {
                                log::error!("Failed to serialize payload: {err:?}");
                                continue;
                            }
                        };
                        let node = handle.state::<Node>();
                        let simplified_operation =
                            match simplify(&node, notification.header, Some(Body::new(&body[..])))
                                .await
                            {
                                Ok(o) => o,
                                Err(err) => {
                                    log::error!("Failed to simplify operation: {err:?}");
                                    continue;
                                }
                            };

                        // let simplified_operation = SimplifiedOperation {
                        //     header: SimplifiedHeader::from(notification.header),
                        //     body: Some(body),
                        // };

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
        });

    #[cfg(not(mobile))]
    {
        builder = builder.menu(|handle| menu::build_menu(handle));
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
