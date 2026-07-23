#![allow(non_snake_case)]

use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::database::backup::BackupEntry;
use crate::store::AppState;

// ─── File import/export ──────────────────────────────────────

#[tauri::command]
pub async fn export_config_to_file(
    #[allow(non_snake_case)] filePath: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        cc_switch_core::commands::import_export::export_config_to_file(&db, &filePath)
    })
    .await
    .map_err(|e| format!("导出配置失败: {e}"))?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_config_from_file(
    #[allow(non_snake_case)] filePath: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        cc_switch_core::commands::import_export::import_config_from_file(&db, &filePath)
    })
    .await
    .map_err(|e| format!("导入配置失败: {e}"))?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_current_providers_live(state: State<'_, AppState>) -> Result<Value, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let app_state = AppState::new(db);
        cc_switch_core::commands::import_export::sync_current_providers_live(&app_state)
    })
    .await
    .map_err(|e| format!("同步当前供应商失败: {e}"))?
    .map_err(|e| e.to_string())
}

// ─── File dialogs ────────────────────────────────────────────

/// 保存文件对话框
#[tauri::command]
pub async fn save_file_dialog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    #[allow(non_snake_case)] defaultName: String,
) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let result = dialog
        .file()
        .add_filter("SQL", &["sql"])
        .set_file_name(&defaultName)
        .blocking_save_file();

    Ok(result.map(|p| p.to_string()))
}

/// 打开文件对话框
#[tauri::command]
pub async fn open_file_dialog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let result = dialog
        .file()
        .add_filter("SQL", &["sql"])
        .blocking_pick_file();

    Ok(result.map(|p| p.to_string()))
}

/// 打开 ZIP 文件选择对话框
#[tauri::command]
pub async fn open_zip_file_dialog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let result = dialog
        .file()
        .add_filter("ZIP / Skill", &["zip", "skill"])
        .blocking_pick_file();

    Ok(result.map(|p| p.to_string()))
}

// ─── Database backup management ─────────────────────────────

#[tauri::command]
pub async fn create_db_backup(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        cc_switch_core::commands::import_export::create_db_backup(&db)
    })
    .await
    .map_err(|e| format!("Backup failed: {e}"))?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_db_backups() -> Result<Vec<BackupEntry>, String> {
    cc_switch_core::commands::import_export::list_db_backups().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_db_backup(
    state: State<'_, AppState>,
    filename: String,
) -> Result<String, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        cc_switch_core::commands::import_export::restore_db_backup(&db, &filename)
    })
    .await
    .map_err(|e| format!("Restore failed: {e}"))?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_db_backup(
    #[allow(non_snake_case)] oldFilename: String,
    #[allow(non_snake_case)] newName: String,
) -> Result<String, String> {
    cc_switch_core::commands::import_export::rename_db_backup(&oldFilename, &newName)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_db_backup(filename: String) -> Result<(), String> {
    cc_switch_core::commands::import_export::delete_db_backup(&filename).map_err(|e| e.to_string())
}
