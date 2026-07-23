//! 代理服务相关的 Tauri 命令
//!
//! 提供前端调用的 API 接口。所有业务逻辑已下沉到 cc-switch-core，本文件仅保留
//! Tauri 薄壳与需要 AppHandle 的 UI 副作用（如 reset_circuit_breaker 的事件发射）。

use crate::error::AppError;
use crate::proxy::types::*;
use crate::proxy::{CircuitBreakerConfig, CircuitBreakerStats};
use crate::store::AppState;

/// 启动代理服务器（仅启动服务，不接管 Live 配置）
#[tauri::command]
pub async fn start_proxy_server(
    state: tauri::State<'_, AppState>,
) -> Result<ProxyServerInfo, String> {
    cc_switch_core::commands::proxy::start_proxy_server(&state.proxy_service)
        .await
        .map_err(|e| e.to_string())
}

/// 停止代理服务器（仅停止服务，不恢复/清理 Live 接管状态）
#[tauri::command]
pub async fn stop_proxy_server(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let takeover =
        cc_switch_core::commands::proxy::get_proxy_takeover_status(&state.proxy_service).await?;
    if takeover.claude
        || takeover.codex
        || takeover.gemini
        || takeover.grokbuild
        || takeover.opencode
        || takeover.openclaw
    {
        return Err(
            "仍有应用处于代理接管状态，请先在设置中关闭对应应用接管后再停止本地路由。".to_string(),
        );
    }

    cc_switch_core::commands::proxy::stop_proxy_server(&state.proxy_service)
        .await
        .map_err(|e| e.to_string())
}

/// 停止代理服务器（恢复 Live 配置）
#[tauri::command]
pub async fn stop_proxy_with_restore(state: tauri::State<'_, AppState>) -> Result<(), String> {
    cc_switch_core::commands::proxy::stop_proxy_with_restore(&state.proxy_service)
        .await
        .map_err(|e| e.to_string())
}

/// 获取各应用接管状态
#[tauri::command]
pub async fn get_proxy_takeover_status(
    state: tauri::State<'_, AppState>,
) -> Result<ProxyTakeoverStatus, String> {
    cc_switch_core::commands::proxy::get_proxy_takeover_status(&state.proxy_service)
        .await
        .map_err(|e| e.to_string())
}

/// 为指定应用开启/关闭接管
#[tauri::command]
pub async fn set_proxy_takeover_for_app(
    state: tauri::State<'_, AppState>,
    app_type: String,
    enabled: bool,
) -> Result<(), String> {
    cc_switch_core::commands::proxy::set_proxy_takeover_for_app(
        &state.proxy_service,
        &app_type,
        enabled,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取代理服务器状态
#[tauri::command]
pub async fn get_proxy_status(state: tauri::State<'_, AppState>) -> Result<ProxyStatus, String> {
    cc_switch_core::commands::proxy::get_proxy_status(&state.proxy_service)
        .await
        .map_err(|e| e.to_string())
}

/// 获取代理配置
#[tauri::command]
pub async fn get_proxy_config(state: tauri::State<'_, AppState>) -> Result<ProxyConfig, String> {
    cc_switch_core::commands::proxy::get_proxy_config(&state.proxy_service)
        .await
        .map_err(|e| e.to_string())
}

/// 更新代理配置
#[tauri::command]
pub async fn update_proxy_config(
    state: tauri::State<'_, AppState>,
    config: ProxyConfig,
) -> Result<(), String> {
    cc_switch_core::commands::proxy::update_proxy_config(&state.proxy_service, config)
        .await
        .map_err(|e| e.to_string())
}

// ==================== Global & Per-App Config ====================

/// 获取全局代理配置
#[tauri::command]
pub async fn get_global_proxy_config(
    state: tauri::State<'_, AppState>,
) -> Result<GlobalProxyConfig, String> {
    cc_switch_core::commands::proxy::get_global_proxy_config(state.inner())
        .await
        .map_err(|e| e.to_string())
}

/// 更新全局代理配置
#[tauri::command]
pub async fn update_global_proxy_config(
    state: tauri::State<'_, AppState>,
    config: GlobalProxyConfig,
) -> Result<(), String> {
    cc_switch_core::commands::proxy::update_global_proxy_config(state.inner(), config)
        .await
        .map_err(|e| e.to_string())
}

/// 获取指定应用的代理配置
#[tauri::command]
pub async fn get_proxy_config_for_app(
    state: tauri::State<'_, AppState>,
    app_type: String,
) -> Result<AppProxyConfig, String> {
    cc_switch_core::commands::proxy::get_proxy_config_for_app(state.inner(), &app_type)
        .await
        .map_err(|e| e.to_string())
}

/// 更新指定应用的代理配置
#[tauri::command]
pub async fn update_proxy_config_for_app(
    state: tauri::State<'_, AppState>,
    config: AppProxyConfig,
) -> Result<(), String> {
    cc_switch_core::commands::proxy::update_proxy_config_for_app(state.inner(), config)
        .await
        .map_err(|e| e.to_string())
}

/// 获取默认成本倍率
#[tauri::command]
pub async fn get_default_cost_multiplier(
    state: tauri::State<'_, AppState>,
    app_type: String,
) -> Result<String, String> {
    cc_switch_core::commands::proxy::get_default_cost_multiplier(state.inner(), &app_type)
        .await
        .map_err(|e| e.to_string())
}

/// 设置默认成本倍率
#[tauri::command]
pub async fn set_default_cost_multiplier(
    state: tauri::State<'_, AppState>,
    app_type: String,
    value: String,
) -> Result<(), String> {
    cc_switch_core::commands::proxy::set_default_cost_multiplier(state.inner(), &app_type, &value)
        .await
        .map_err(|e| e.to_string())
}

/// 获取计费模式来源
#[tauri::command]
pub async fn get_pricing_model_source(
    state: tauri::State<'_, AppState>,
    app_type: String,
) -> Result<String, String> {
    cc_switch_core::commands::proxy::get_pricing_model_source(state.inner(), &app_type)
        .await
        .map_err(|e| e.to_string())
}

/// 设置计费模式来源
#[tauri::command]
pub async fn set_pricing_model_source(
    state: tauri::State<'_, AppState>,
    app_type: String,
    value: String,
) -> Result<(), String> {
    cc_switch_core::commands::proxy::set_pricing_model_source(state.inner(), &app_type, &value)
        .await
        .map_err(|e| e.to_string())
}

/// 检查代理服务器是否正在运行
#[tauri::command]
pub async fn is_proxy_running(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(cc_switch_core::commands::proxy::is_proxy_running(&state.proxy_service).await)
}

/// 检查是否处于 Live 接管模式
#[tauri::command]
pub async fn is_live_takeover_active(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    cc_switch_core::commands::proxy::is_live_takeover_active(&state.proxy_service)
        .await
        .map_err(|e| e.to_string())
}

/// 代理模式下切换供应商（热切换）
#[tauri::command]
pub async fn switch_proxy_provider(
    state: tauri::State<'_, AppState>,
    app_type: String,
    provider_id: String,
) -> Result<(), String> {
    cc_switch_core::commands::proxy::switch_proxy_provider(state.inner(), &app_type, &provider_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取供应商健康状态
#[tauri::command]
pub async fn get_provider_health(
    state: tauri::State<'_, AppState>,
    provider_id: String,
    app_type: String,
) -> Result<ProviderHealth, String> {
    cc_switch_core::commands::proxy::get_provider_health(state.inner(), &provider_id, &app_type)
        .await
        .map_err(|e| e.to_string())
}

/// 重置熔断器
///
/// 重置后会检查是否应该切回队列中优先级更高的供应商。
/// 该命令保留在 tauri 层，因为需要发射 `provider-switched` 事件。
#[tauri::command]
pub async fn reset_circuit_breaker(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    provider_id: String,
    app_type: String,
) -> Result<(), String> {
    let db = &state.db;
    db.update_provider_health(&provider_id, &app_type, true, None)
        .await
        .map_err(|e| e.to_string())?;

    state
        .proxy_service
        .reset_provider_circuit_breaker(&provider_id, &app_type)
        .await?;

    let (app_enabled, auto_failover_enabled) = match db.get_proxy_config_for_app(&app_type).await {
        Ok(config) => (config.enabled, config.auto_failover_enabled),
        Err(e) => {
            log::error!("[{app_type}] Failed to read proxy_config: {e}, defaulting to disabled");
            (false, false)
        }
    };

    if app_enabled && auto_failover_enabled && state.proxy_service.is_running().await {
        let current_id = db
            .get_current_provider(&app_type)
            .map_err(|e| e.to_string())?;

        if let Some(current_id) = current_id {
            let queue = db
                .get_failover_queue(&app_type)
                .map_err(|e| e.to_string())?;

            let restored_order = queue
                .iter()
                .find(|item| item.provider_id == provider_id)
                .and_then(|item| item.sort_index);

            let current_order = queue
                .iter()
                .find(|item| item.provider_id == current_id)
                .and_then(|item| item.sort_index);

            if let (Some(restored), Some(current)) = (restored_order, current_order) {
                if restored < current {
                    log::info!(
                        "[Recovery] 供应商 {provider_id} 已恢复且优先级更高 (P{restored} vs P{current})，自动切换"
                    );

                    let provider_name = db
                        .get_all_providers(&app_type)
                        .ok()
                        .and_then(|providers| providers.get(&provider_id).map(|p| p.name.clone()))
                        .unwrap_or_else(|| provider_id.clone());

                    let switch_manager =
                        crate::proxy::failover_switch::FailoverSwitchManager::new(db.clone());
                    if let Err(e) = switch_manager
                        .try_switch(Some(&app_handle), &app_type, &provider_id, &provider_name)
                        .await
                    {
                        log::error!("[Recovery] 自动切换失败: {e}");
                    }
                }
            }
        }
    }

    Ok(())
}

/// 获取熔断器配置
#[tauri::command]
pub async fn get_circuit_breaker_config(
    state: tauri::State<'_, AppState>,
) -> Result<CircuitBreakerConfig, String> {
    cc_switch_core::commands::proxy::get_circuit_breaker_config(state.inner())
        .await
        .map_err(|e| e.to_string())
}

/// 更新熔断器配置
#[tauri::command]
pub async fn update_circuit_breaker_config(
    state: tauri::State<'_, AppState>,
    config: CircuitBreakerConfig,
) -> Result<(), String> {
    cc_switch_core::commands::proxy::update_circuit_breaker_config(state.inner(), config)
        .await
        .map_err(|e| e.to_string())
}

/// 获取熔断器统计信息（仅当代理服务器运行时）
#[tauri::command]
pub async fn get_circuit_breaker_stats(
    state: tauri::State<'_, AppState>,
    provider_id: String,
    app_type: String,
) -> Result<Option<CircuitBreakerStats>, String> {
    cc_switch_core::commands::proxy::get_circuit_breaker_stats(
        state.inner(),
        &provider_id,
        &app_type,
    )
    .await
    .map_err(|e| e.to_string())
}
