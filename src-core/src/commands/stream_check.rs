//! 供应商连通性检查命令。
//!
//! 完整实现：探测单个 / 所有供应商，依赖 `StreamCheckService` 与
//! `CopilotAuthManager`（用于解析 Copilot 的动态 base_url）。

use std::collections::HashSet;

use crate::app_config::AppType;
use crate::database::Database;
use crate::error::AppError;
use crate::provider::Provider;
use crate::services::stream_check::{StreamCheckConfig, StreamCheckResult, StreamCheckService};
use crate::store::AppState;

/// 获取连通性检查配置。
pub fn get_stream_check_config(db: &Database) -> Result<StreamCheckConfig, AppError> {
    db.get_stream_check_config()
}

/// 保存连通性检查配置。
pub fn save_stream_check_config(
    db: &Database,
    config: StreamCheckConfig,
) -> Result<(), AppError> {
    db.save_stream_check_config(&config)
}

/// 检查单个供应商的连通性，并将结果写入 `stream_check_logs` 表。
pub async fn stream_check_provider(
    state: &AppState,
    app_type: AppType,
    provider_id: &str,
) -> Result<StreamCheckResult, AppError> {
    let config = state.db.get_stream_check_config()?;
    let providers = state.db.get_all_providers(app_type.as_str())?;
    let provider = providers
        .get(provider_id)
        .ok_or_else(|| AppError::Message(format!("供应商 {provider_id} 不存在")))?;

    let base_url_override = resolve_copilot_base_url_override(state, provider).await?;
    let result = StreamCheckService::check_with_retry(&app_type, provider, &config, base_url_override)
        .await?;

    let _ = state.db.save_stream_check_log(
        provider_id,
        &provider.name,
        app_type.as_str(),
        &result,
    );
    Ok(result)
}

/// 检查指定应用下所有供应商的连通性。
///
/// `proxy_targets_only` 为 true 时只检查当前供应商 + 故障转移队列中的供应商。
pub async fn stream_check_all_providers(
    state: &AppState,
    app_type: AppType,
    proxy_targets_only: bool,
) -> Result<Vec<(String, StreamCheckResult)>, AppError> {
    let config = state.db.get_stream_check_config()?;
    let providers = state.db.get_all_providers(app_type.as_str())?;

    let allowed_ids: Option<HashSet<String>> = if proxy_targets_only {
        let mut ids = HashSet::new();
        if let Ok(Some(current_id)) = state.db.get_current_provider(app_type.as_str()) {
            ids.insert(current_id);
        }
        if let Ok(queue) = state.db.get_failover_queue(app_type.as_str()) {
            for item in queue {
                ids.insert(item.provider_id);
            }
        }
        Some(ids)
    } else {
        None
    };

    let mut results = Vec::new();
    for (id, provider) in providers {
        if let Some(ref allowed) = allowed_ids {
            if !allowed.contains(&id) {
                continue;
            }
        }
        let base_url_override = resolve_copilot_base_url_override(state, &provider).await?;
        let result = StreamCheckService::check_with_retry(&app_type, &provider, &config, base_url_override)
            .await
            .unwrap_or_else(|e| StreamCheckResult {
                status: crate::services::stream_check::HealthStatus::Failed,
                success: false,
                message: e.to_string(),
                response_time_ms: None,
                http_status: None,
                model_used: String::new(),
                tested_at: chrono::Utc::now().timestamp(),
                retry_count: 0,
                error_category: None,
            });
        let _ = state
            .db
            .save_stream_check_log(&id, &provider.name, app_type.as_str(), &result);
        results.push((id, result));
    }
    Ok(results)
}

/// Copilot 端点是动态的（随 OAuth token 解析），需预先取出 host 再探测；
/// 其余供应商返回 None，由服务层从 settings_config 提取 base_url。
async fn resolve_copilot_base_url_override(
    state: &AppState,
    provider: &Provider,
) -> Result<Option<String>, AppError> {
    let is_copilot = is_copilot_provider(provider);
    let is_full_url = provider
        .meta
        .as_ref()
        .and_then(|meta| meta.is_full_url)
        .unwrap_or(false);

    if !is_copilot || is_full_url {
        return Ok(None);
    }

    let copilot_arc = state.proxy_service.auth_state().copilot.clone();
    let auth_manager = copilot_arc.read().await;
    let account_id = provider
        .meta
        .as_ref()
        .and_then(|meta| meta.managed_account_id_for("github_copilot"));

    let endpoint = match account_id.as_deref() {
        Some(id) => auth_manager.get_api_endpoint(id).await,
        None => auth_manager.get_default_api_endpoint().await,
    };

    Ok(Some(endpoint))
}

fn is_copilot_provider(provider: &Provider) -> bool {
    provider
        .meta
        .as_ref()
        .and_then(|meta| meta.provider_type.as_deref())
        == Some("github_copilot")
}
