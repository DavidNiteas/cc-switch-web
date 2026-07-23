use crate::app_config::AppType;
use crate::error::AppError;
use crate::proxy::types::{
    AppProxyConfig, CircuitBreakerConfig, CircuitBreakerStats, GlobalProxyConfig, ProxyConfig,
    ProxyStatus, ProxyTakeoverStatus,
};
use crate::proxy::types::{ProviderHealth, ProxyServerInfo};
use crate::services::provider::official_provider_supports_proxy_takeover;
use crate::services::ProxyService;
use crate::store::AppState;
use std::str::FromStr;

/// 获取本地代理服务器当前状态。
pub async fn get_proxy_status(proxy_service: &ProxyService) -> Result<ProxyStatus, AppError> {
    proxy_service
        .get_status()
        .await
        .map_err(|e| AppError::Message(format!("获取代理状态失败: {e}")))
}

/// 启动本地代理服务器。
pub async fn start_proxy_server(proxy_service: &ProxyService) -> Result<ProxyServerInfo, AppError> {
    proxy_service
        .start()
        .await
        .map_err(|e| AppError::Message(format!("启动代理服务器失败: {e}")))
}

/// 停止本地代理服务器。
pub async fn stop_proxy_server(proxy_service: &ProxyService) -> Result<(), AppError> {
    proxy_service
        .stop()
        .await
        .map_err(|e| AppError::Message(format!("停止代理服务器失败: {e}")))
}

/// 停止代理服务器并恢复 Live 配置。
pub async fn stop_proxy_with_restore(proxy_service: &ProxyService) -> Result<(), AppError> {
    proxy_service
        .stop_with_restore()
        .await
        .map_err(|e| AppError::Message(format!("停止并恢复代理失败: {e}")))
}

/// 获取各应用接管状态。
pub async fn get_proxy_takeover_status(
    proxy_service: &ProxyService,
) -> Result<ProxyTakeoverStatus, AppError> {
    proxy_service
        .get_takeover_status()
        .await
        .map_err(|e| AppError::Message(format!("获取接管状态失败: {e}")))
}

/// 为指定应用开启/关闭接管。
pub async fn set_proxy_takeover_for_app(
    proxy_service: &ProxyService,
    app_type: &str,
    enabled: bool,
) -> Result<(), AppError> {
    proxy_service
        .set_takeover_for_app(app_type, enabled)
        .await
        .map_err(|e| AppError::Message(format!("设置接管状态失败: {e}")))
}

/// 获取代理配置。
pub async fn get_proxy_config(proxy_service: &ProxyService) -> Result<ProxyConfig, AppError> {
    proxy_service
        .get_config()
        .await
        .map_err(|e| AppError::Message(format!("获取代理配置失败: {e}")))
}

/// 更新代理配置。
pub async fn update_proxy_config(
    proxy_service: &ProxyService,
    config: ProxyConfig,
) -> Result<(), AppError> {
    proxy_service
        .update_config(&config)
        .await
        .map_err(|e| AppError::Message(format!("更新代理配置失败: {e}")))
}

/// 获取全局代理配置。
pub async fn get_global_proxy_config(state: &AppState) -> Result<GlobalProxyConfig, AppError> {
    state.db.get_global_proxy_config().await
}

/// 更新全局代理配置。
pub async fn update_global_proxy_config(
    state: &AppState,
    config: GlobalProxyConfig,
) -> Result<(), AppError> {
    state.db.update_global_proxy_config(config).await
}

/// 获取指定应用的代理配置。
pub async fn get_proxy_config_for_app(
    state: &AppState,
    app_type: &str,
) -> Result<AppProxyConfig, AppError> {
    state.db.get_proxy_config_for_app(app_type).await
}

/// 更新指定应用的代理配置。
pub async fn update_proxy_config_for_app(
    state: &AppState,
    config: AppProxyConfig,
) -> Result<(), AppError> {
    let app_type = config.app_type.clone();
    let circuit_config = CircuitBreakerConfig {
        failure_threshold: config.circuit_failure_threshold,
        success_threshold: config.circuit_success_threshold,
        timeout_seconds: config.circuit_timeout_seconds as u64,
        error_rate_threshold: config.circuit_error_rate_threshold,
        min_requests: config.circuit_min_requests,
    };

    state.db.update_proxy_config_for_app(config).await?;
    state
        .proxy_service
        .update_circuit_breaker_config_for_app(&app_type, circuit_config)
        .await
        .map_err(|e| AppError::Message(format!("热更新熔断器配置失败: {e}")))
}

/// 获取默认成本倍率。
pub async fn get_default_cost_multiplier(
    state: &AppState,
    app_type: &str,
) -> Result<String, AppError> {
    state.db.get_default_cost_multiplier(app_type).await
}

/// 设置默认成本倍率。
pub async fn set_default_cost_multiplier(
    state: &AppState,
    app_type: &str,
    value: &str,
) -> Result<(), AppError> {
    state.db.set_default_cost_multiplier(app_type, value).await
}

/// 获取计费模式来源。
pub async fn get_pricing_model_source(
    state: &AppState,
    app_type: &str,
) -> Result<String, AppError> {
    state.db.get_pricing_model_source(app_type).await
}

/// 设置计费模式来源。
pub async fn set_pricing_model_source(
    state: &AppState,
    app_type: &str,
    value: &str,
) -> Result<(), AppError> {
    state.db.set_pricing_model_source(app_type, value).await
}

/// 检查代理服务器是否正在运行。
pub async fn is_proxy_running(proxy_service: &ProxyService) -> bool {
    proxy_service.is_running().await
}

/// 检查是否处于 Live 接管模式。
pub async fn is_live_takeover_active(proxy_service: &ProxyService) -> Result<bool, AppError> {
    proxy_service
        .is_takeover_active()
        .await
        .map_err(|e| AppError::Message(format!("检查接管状态失败: {e}")))
}

/// 代理模式下切换供应商（热切换）。
pub async fn switch_proxy_provider(
    state: &AppState,
    app_type: &str,
    provider_id: &str,
) -> Result<(), AppError> {
    let provider = state
        .db
        .get_provider_by_id(provider_id, app_type)
        .map_err(|e| AppError::Message(format!("读取供应商失败: {e}")))?
        .ok_or_else(|| AppError::Message(format!("供应商不存在: {provider_id}")))?;

    let app = AppType::from_str(app_type)
        .map_err(|e| AppError::Message(format!("无效的应用类型: {e}")))?;

    if provider.category.as_deref() == Some("official")
        && !official_provider_supports_proxy_takeover(&app, &provider)
    {
        return Err(AppError::Message(
            "代理接管模式下不能切换到官方供应商 (Cannot switch to official provider during proxy takeover)"
                .to_string(),
        ));
    }

    state
        .proxy_service
        .switch_proxy_target(app_type, provider_id)
        .await
        .map_err(|e| AppError::Message(format!("热切换供应商失败: {e}")))
}

/// 获取供应商健康状态。
pub async fn get_provider_health(
    state: &AppState,
    provider_id: &str,
    app_type: &str,
) -> Result<ProviderHealth, AppError> {
    state.db.get_provider_health(provider_id, app_type).await
}

/// 获取熔断器配置。
pub async fn get_circuit_breaker_config(
    state: &AppState,
) -> Result<CircuitBreakerConfig, AppError> {
    state.db.get_circuit_breaker_config().await
}

/// 更新熔断器配置。
pub async fn update_circuit_breaker_config(
    state: &AppState,
    config: CircuitBreakerConfig,
) -> Result<(), AppError> {
    state.db.update_circuit_breaker_config(&config).await?;
    state
        .proxy_service
        .update_circuit_breaker_configs(config)
        .await
        .map_err(|e| AppError::Message(format!("热更新熔断器配置失败: {e}")))
}

/// 获取熔断器统计信息（仅当代理服务器运行时）。
pub async fn get_circuit_breaker_stats(
    _state: &AppState,
    _provider_id: &str,
    _app_type: &str,
) -> Result<Option<CircuitBreakerStats>, AppError> {
    // 该功能需要访问运行中代理服务器的内存状态；目前返回 None，后续可通过 ProxyService 暴露接口实现。
    Ok(None)
}

/// 重置指定 provider 的熔断器状态。
///
/// 业务逻辑：① 更新 provider_health 表；② 重置 ProxyService 内的熔断器；
/// ③ 若启用 auto-failover 且代理运行中、被恢复的 provider 优先级高于当前，
/// 自动切换到该 provider。
///
/// UI 副作用：自动切换会通过 `Platform::emit_event` 发射 `provider-switched`
/// 事件。Web 模式下 platform 通常为 `HeadlessPlatform`，emit 会推送到 SSE
/// 广播；桌面版由 `TauriPlatform` 走 `AppHandle::emit`。
pub async fn reset_circuit_breaker(
    state: &AppState,
    provider_id: &str,
    app_type: &str,
) -> Result<(), AppError> {
    let db = &state.db;
    db.update_provider_health(provider_id, app_type, true, None)
        .await?;
    state
        .proxy_service
        .reset_provider_circuit_breaker(provider_id, app_type)
        .await
        .map_err(AppError::Message)?;

    let (app_enabled, auto_failover_enabled) = match db.get_proxy_config_for_app(app_type).await {
        Ok(config) => (config.enabled, config.auto_failover_enabled),
        Err(e) => {
            log::error!("[{app_type}] Failed to read proxy_config: {e}, defaulting to disabled");
            (false, false)
        }
    };

    if app_enabled && auto_failover_enabled && state.proxy_service.is_running().await {
        let current_id = db.get_current_provider(app_type)?;
        if let Some(current_id) = current_id {
            let queue = db.get_failover_queue(app_type)?;
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
                        .get_all_providers(app_type)
                        .ok()
                        .and_then(|providers| providers.get(provider_id).map(|p| p.name.clone()))
                        .unwrap_or_else(|| provider_id.to_string());

                    let switch_manager =
                        crate::proxy::failover_switch::FailoverSwitchManager::new(db.clone());
                    let platform = state.proxy_service.platform().await;
                    if let Err(e) = switch_manager
                        .try_switch(platform.as_ref(), app_type, provider_id, &provider_name)
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
