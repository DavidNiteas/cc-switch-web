#![allow(non_snake_case)]

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::app_config::AppType;
use crate::config::ConfigStatus;
use crate::store::AppState;

#[tauri::command]
pub async fn get_claude_config_status() -> Result<ConfigStatus, String> {
    cc_switch_core::commands::config::get_claude_config_status().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_config_status(
    state: State<'_, AppState>,
    app: String,
) -> Result<ConfigStatus, String> {
    let proxy_running = state.proxy_service.is_running().await;
    cc_switch_core::commands::config::get_config_status(&state.db, &app, proxy_running)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_claude_code_config_path() -> Result<String, String> {
    cc_switch_core::commands::config::get_claude_code_config_path().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_config_dir(app: String) -> Result<String, String> {
    cc_switch_core::commands::config::get_config_dir(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_config_folder(handle: AppHandle, app: String) -> Result<bool, String> {
    let config_dir =
        cc_switch_core::commands::config::get_config_dir(&app).map_err(|e| e.to_string())?;
    let config_dir = std::path::PathBuf::from(config_dir);

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    handle
        .opener()
        .open_path(config_dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| format!("打开文件夹失败: {e}"))?;

    Ok(true)
}

#[tauri::command]
pub async fn pick_directory(
    app: AppHandle,
    #[allow(non_snake_case)] defaultPath: Option<String>,
) -> Result<Option<String>, String> {
    let initial = defaultPath
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut builder = app.dialog().file();
        if let Some(path) = initial {
            builder = builder.set_directory(path);
        }
        builder.blocking_pick_folder()
    })
    .await
    .map_err(|e| format!("弹出目录选择器失败: {e}"))?;

    match result {
        Some(file_path) => {
            let resolved = file_path
                .simplified()
                .into_path()
                .map_err(|e| format!("解析选择的目录失败: {e}"))?;
            Ok(Some(resolved.to_string_lossy().to_string()))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_app_config_path() -> Result<String, String> {
    cc_switch_core::commands::config::get_app_config_path().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_app_config_folder(handle: AppHandle) -> Result<bool, String> {
    let config_dir =
        cc_switch_core::commands::config::get_app_config_path().map_err(|e| e.to_string())?;
    let config_dir = std::path::Path::new(&config_dir)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from(&config_dir));

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    handle
        .opener()
        .open_path(config_dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| format!("打开文件夹失败: {e}"))?;

    Ok(true)
}

#[tauri::command]
pub async fn get_claude_common_config_snippet(
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    cc_switch_core::commands::config::get_claude_common_config_snippet(&state.db)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_claude_common_config_snippet(
    snippet: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    cc_switch_core::commands::config::set_claude_common_config_snippet(&state.db, &snippet)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_common_config_snippet(
    app_type: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    cc_switch_core::commands::config::get_common_config_snippet(&state.db, &app_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_toml_common_config_snippet(
    config_toml: String,
    snippet_toml: String,
    enabled: bool,
) -> Result<String, String> {
    cc_switch_core::commands::config::update_toml_common_config_snippet(
        &config_toml,
        &snippet_toml,
        enabled,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_common_config_snippet(
    app_type: String,
    snippet: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    cc_switch_core::commands::config::set_common_config_snippet(&state, &app_type, &snippet)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_common_config_snippet(
    appType: String,
    settingsConfig: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    cc_switch_core::commands::config::extract_common_config_snippet(
        &state,
        &appType,
        settingsConfig.as_deref(),
    )
    .map_err(|e| e.to_string())
}
