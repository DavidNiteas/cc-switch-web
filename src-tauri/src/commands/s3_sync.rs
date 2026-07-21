#![allow(non_snake_case)]

use serde_json::Value;
use tauri::State;

use crate::store::AppState;

#[tauri::command]
pub async fn s3_test_connection(
    settings: cc_switch_core::settings::S3SyncSettings,
    #[allow(non_snake_case)] preserveEmptyPassword: Option<bool>,
) -> Result<Value, String> {
    cc_switch_core::commands::s3_sync::s3_test_connection(settings, preserveEmptyPassword).await
}

#[tauri::command]
pub async fn s3_sync_upload(state: State<'_, AppState>) -> Result<Value, String> {
    cc_switch_core::commands::s3_sync::s3_sync_upload(&*state).await
}

#[tauri::command]
pub async fn s3_sync_download(state: State<'_, AppState>) -> Result<Value, String> {
    cc_switch_core::commands::s3_sync::s3_sync_download(&*state).await
}

#[tauri::command]
pub async fn s3_sync_save_settings(
    settings: cc_switch_core::settings::S3SyncSettings,
    #[allow(non_snake_case)] passwordTouched: Option<bool>,
) -> Result<Value, String> {
    cc_switch_core::commands::s3_sync::s3_sync_save_settings(settings, passwordTouched).await
}

#[tauri::command]
pub async fn s3_sync_fetch_remote_info() -> Result<Value, String> {
    cc_switch_core::commands::s3_sync::s3_sync_fetch_remote_info().await
}
