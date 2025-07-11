use std::collections::HashMap;

use holochain_client::ZomeCallTarget;
use holochain_types::prelude::ZomeName;
use jni::objects::JClass;
use jni::JNIEnv;
use notifications_zome_trait::GetNotificationInput;
use push_notifications_service_trait::*;
use service_providers_utils::make_service_request;
use tauri::{AppHandle, Listener, Manager};
use tauri_plugin_holochain::*;
use tauri_plugin_notification::{NotificationData, NotificationExt};

use crate::{app_id, holochain_dir, network_config, open_window, utils::with_retries};

pub fn setup_push_notifications(handle: AppHandle) -> anyhow::Result<()> {
    let h = handle.clone();
    handle.listen("notification://new-fcm-token", move |event| {
        if let Ok(token) = serde_json::from_str::<String>(event.payload()) {
            log::warn!("New FCM token: {:?}. Registering it in with the push notifications service_providers.", token);
            let h = h.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = register_fcm_token(h, token.clone()).await {
                    log::error!("Error registering FCM token: {:?}", err);
                } else {
                    log::info!("Successfully registered FCM token.");
                }
            });
        }
    });

    let h = handle.clone();
    handle.listen("notification://action-performed", move |event| {
        let handle = h.clone();
        if let Ok(notification_action_performed_payload) = serde_json::from_str::<
            tauri_plugin_notification::NotificationActionPerformedPayload,
        >(event.payload())
        {
            tauri::async_runtime::spawn(async move {
                println!("onlistener{:?}", notification_action_performed_payload);
                let window = match handle.get_webview_window("main") {
                    Some(w) => w,
                    None => open_window(handle.clone())
                        .await
                        .expect("Failed to open window"),
                };

                let extra: HashMap<String, serde_json::Value> = notification_action_performed_payload.notification.extra;
                let Some(url_to_navigate_to_on_click) = extra.get("url_path_to_navigate_to_on_click") else {
                    log::error!("Notification did not have the url_path_to_navigate_to_on_click extra.");
                    return;
                };

                let Ok(url_to_navigate_to_on_click)  = serde_json::from_value::<Option<String>>(url_to_navigate_to_on_click.clone()) else {
                    log::error!("Notification's url_path_to_navigate_to_on_click extra failed to deserialize.");
                    return;
                };

                let Some(url) = url_to_navigate_to_on_click else {
                    return;
                };
                let Ok(url) = tauri::Url::parse(url.as_str()) else {
                    log::error!("Failed to parse url for notification.");
                    return;
                };

                if let Err(err) = window.navigate(url) {
                    log::error!("Failed to navigate to url: {err:?}");
                }
            });
        }
    });

    Ok(())
}

async fn register_fcm_token(handle: AppHandle, token: String) -> anyhow::Result<()> {
    let app_ws = handle.holochain()?.app_websocket(app_id()).await?;
    let fcm_project_id = handle.notification().fcm_project_id()?;

    with_retries(
        async move || {
            let _r: () = make_service_request(
                &app_ws,
                PUSH_NOTIFICATIONS_SERVICE_HASH.to_vec(),
                "register_fcm_token".into(),
                RegisterFcmTokenInput {
                    fcm_project_id: fcm_project_id.clone(),
                    token: token.clone(),
                },
            )
            .await?;
            Ok(())
        },
        60,
    )
    .await?;

    Ok(())
}

// Entry point to receive notifications
#[tauri_plugin_notification::receive_push_notification]
pub fn receive_push_notification(notification: NotificationData) -> Option<NotificationData> {
    log::info!("Received push notification: {:?}.", notification);

    tauri::async_runtime::block_on(async move {
        let (Some(title), Some(body)) = (notification.title.clone(), notification.body.clone())
        else {
            log::warn!("Received a push notification without title or body: {notification:?}.");
            return None;
        };
        let zome_name = ZomeName::from(title);
        let notification_id = body;
        let Ok(notification) = get_notification(zome_name, notification_id).await else {
            log::error!("Failed to get notifications.");
            return None;
        };
        if let Some(n) = &notification {
            log::info!("Showing notification: {:?}.", n);
        } else {
            log::info!("Notification was not necessary to display.");
        }
        notification
    })
}

async fn get_notification(
    zome_name: ZomeName,
    notification_id: String,
) -> anyhow::Result<Option<NotificationData>> {
    let runtime = tauri_plugin_holochain::launch_holochain_runtime(
        vec_to_locked(vec![]),
        HolochainPluginConfig::new(holochain_dir(), network_config()),
    )
    .await?;

    let app_ws = runtime.app_websocket(app_id(), AllowedOrigins::Any).await?;

    log::debug!("[receive_push_notification] Calling receive messages.");

    app_ws
        .call_zome(
            ZomeCallTarget::RoleName("main".into()),
            "safehold_async_messages".into(),
            "receive_messages".into(),
            ExternIO::encode(())?,
        )
        .await?;

    log::debug!("[receive_push_notification] Calling get_notification.");

    let notification: Option<notifications_zome_trait::Notification> = app_ws
        .call_zome(
            ZomeCallTarget::RoleName("main".into()),
            zome_name,
            "get_notification".into(),
            ExternIO::encode(GetNotificationInput {
                notification_id,
                locale: String::from("en-US"),
            })?,
        )
        .await?
        .decode()?;

    let Some(notification) = notification else {
        return Ok(None);
    };

    let extra = match notification.url_path_to_navigate_to_on_click {
        Some(url) => vec![(
            String::from("url_path_to_navigate_to_on_click"),
            serde_json::to_value(url)?,
        )]
        .into_iter()
        .collect(),
        None => HashMap::new(),
    };

    Ok(Some(NotificationData {
        title: Some(notification.title),
        body: Some(notification.body),
        group: notification.group,
        extra,
        ..Default::default()
    }))
}
