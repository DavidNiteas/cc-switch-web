//! 全局出站代理相关命令
//!
//! 提供获取、设置和测试全局代理的 Tauri 命令。

use crate::store::AppState;

/// 获取全局代理 URL
#[tauri::command]
pub fn get_global_proxy_url(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    cc_switch_core::commands::global_proxy::get_global_proxy_url(&state.db)
        .map_err(|e| e.to_string())
}

/// 设置全局代理 URL
#[tauri::command]
pub fn set_global_proxy_url(state: tauri::State<'_, AppState>, url: String) -> Result<(), String> {
    cc_switch_core::commands::global_proxy::set_global_proxy_url(&state.db, &url)
        .map_err(|e| e.to_string())
}

/// 测试代理连接
#[tauri::command]
pub async fn test_proxy_url(
    url: String,
) -> Result<cc_switch_core::commands::global_proxy::ProxyTestResult, String> {
    cc_switch_core::commands::global_proxy::test_proxy_url(&url)
        .await
        .map_err(|e| e.to_string())
}

/// 获取当前出站代理状态
#[tauri::command]
pub fn get_upstream_proxy_status() -> cc_switch_core::commands::global_proxy::UpstreamProxyStatus {
    cc_switch_core::commands::global_proxy::get_upstream_proxy_status()
}

/// 扫描本地常见代理端口
#[tauri::command]
pub async fn scan_local_proxies() -> Vec<cc_switch_core::commands::global_proxy::DetectedProxy> {
    cc_switch_core::commands::global_proxy::scan_local_proxies().await
}
