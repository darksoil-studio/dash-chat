use tauri_plugin_notification::NotificationData;

use jni::objects::JClass;
use jni::JNIEnv;

#[tauri_plugin_notification::modify_push_notification]
pub fn modify_push_notification(notification: NotificationData) -> NotificationData {
    notification
}
