use push_notifications_service_trait::*;
use service_providers_utils::make_service_request;
use tauri::{AppHandle, Listener};
use tauri_plugin_holochain::HolochainExt;
use tauri_plugin_notification::{NotificationExt, PermissionState};

use crate::app_id;

pub fn setup_push_notifications(handle: AppHandle) -> anyhow::Result<()> {
    let h = handle.clone();
    handle.listen("notification://new-fcm-token", move |event| {
        if let Ok(token) = serde_json::from_str::<String>(event.payload()) {
            log::warn!("New FCM token: {:?}. Registering it in with the push notifications service_providers.", token);
            let h = h.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = register_fcm_token(h, token).await {
                    log::error!("Error registering FCM token: {:?}", err);
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

    let _r: () = make_service_request(
        &app_ws,
        PUSH_NOTIFICATIONS_SERVICE_HASH.to_vec(),
        "register_fcm_token".into(),
        RegisterFcmTokenInput {
            fcm_project_id,
            token,
        },
    )
    .await?;

    Ok(())
}
