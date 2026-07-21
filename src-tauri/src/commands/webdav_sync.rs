#![allow(non_snake_case)]

use serde_json::Value;
use tauri::State;

use crate::store::AppState;

#[tauri::command]
pub async fn webdav_test_connection(
    settings: cc_switch_core::settings::WebDavSyncSettings,
    #[allow(non_snake_case)] preserveEmptyPassword: Option<bool>,
) -> Result<Value, String> {
    cc_switch_core::commands::webdav_sync::webdav_test_connection(settings, preserveEmptyPassword)
        .await
}

#[tauri::command]
pub async fn webdav_sync_upload(state: State<'_, AppState>) -> Result<Value, String> {
    cc_switch_core::commands::webdav_sync::webdav_sync_upload(&*state).await
}

#[tauri::command]
pub async fn webdav_sync_download(state: State<'_, AppState>) -> Result<Value, String> {
    cc_switch_core::commands::webdav_sync::webdav_sync_download(&*state).await
}

#[tauri::command]
pub async fn webdav_sync_save_settings(
    settings: cc_switch_core::settings::WebDavSyncSettings,
    #[allow(non_snake_case)] passwordTouched: Option<bool>,
) -> Result<Value, String> {
    cc_switch_core::commands::webdav_sync::webdav_sync_save_settings(settings, passwordTouched)
        .await
}

#[tauri::command]
pub async fn webdav_sync_fetch_remote_info() -> Result<Value, String> {
    cc_switch_core::commands::webdav_sync::webdav_sync_fetch_remote_info().await
}
