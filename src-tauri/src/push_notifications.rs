use jni::objects::JClass;
use jni::JNIEnv;
use push_notifications_service_trait::*;
use service_providers_utils::make_service_request;
use tauri::{AppHandle, Listener};
use tauri_plugin_holochain::*;
use tauri_plugin_notification::{NotificationData, NotificationExt, PermissionState};

use crate::{app_id, holochain_dir, network_config, utils::with_retries};

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

    handle.listen("notification://action-performed", move |event| {
        if let Ok(notification_action_performed_payload) = serde_json::from_str::<
            tauri_plugin_notification::NotificationActionPerformedPayload,
        >(event.payload())
        {
            println!("onlistener{:?}", notification_action_performed_payload);
        }
    });

    match handle.notification().permission_state().unwrap() {
        PermissionState::Prompt | PermissionState::PromptWithRationale => {
            handle.notification().request_permission().unwrap();
        }
        _ => {}
    }

    handle.notification().register_for_push_notifications()?;

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
#[tauri_plugin_notification::modify_push_notification]
pub fn modify_push_notification(notification: NotificationData) -> Vec<NotificationData> {
    tauri::async_runtime::block_on(async move {
        // let Ok(notifications) = get_notifications().await else {
        //     log::error!("Failed to get notifications.");
        //     return vec![notification];
        // };
        vec![notification]
    })
}

async fn get_notifications() -> anyhow::Result<Vec<NotificationData>> {
    let runtime = tauri_plugin_holochain::launch_holochain_runtime(
        vec_to_locked(vec![]),
        HolochainPluginConfig::new(holochain_dir(), network_config()),
    )
    .await?;
    Ok(vec![])
}
