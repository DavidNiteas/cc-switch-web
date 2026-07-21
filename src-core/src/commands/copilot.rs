//! GitHub Copilot 认证命令层（B 类，依赖 ProxyAuthState 单例）。
//!
//! 对应 tauri 侧 `commands/copilot.rs` 的命令，但通过 `AppState.proxy_service`
//! 获取共享的 `CopilotAuthManager`。所有命令转发到 manager 实现。

use crate::error::AppError;
use crate::proxy::providers::copilot_auth::{
    CopilotAuthError, CopilotAuthManager, CopilotAuthStatus, CopilotModel, CopilotUsageResponse,
    GitHubAccount, GitHubDeviceCodeResponse,
};
use crate::proxy::state::ProxyAuthState;
use crate::store::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 从 AppState 获取 Copilot AuthManager 的共享句柄。
fn copilot_manager(state: &AppState) -> Arc<RwLock<CopilotAuthManager>> {
    state.proxy_service.auth_state().copilot.clone()
}

/// 启动设备码 OAuth 流程。
pub async fn copilot_start_device_flow(
    state: &AppState,
    github_domain: Option<&str>,
) -> Result<GitHubDeviceCodeResponse, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    manager
        .start_device_flow(github_domain)
        .await
        .map_err(map_auth_error)
}

/// 轮询设备码授权结果（向后兼容：返回 bool）。
pub async fn copilot_poll_for_auth(
    state: &AppState,
    device_code: &str,
    github_domain: Option<&str>,
) -> Result<bool, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.write().await;
    match manager.poll_for_token(device_code, github_domain).await {
        Ok(Some(_account)) => {
            log::info!("[CopilotAuth] 用户已授权");
            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(CopilotAuthError::AuthorizationPending) => Ok(false),
        Err(e) => {
            log::error!("[CopilotAuth] 轮询失败: {e}");
            Err(map_auth_error(e))
        }
    }
}

/// 轮询设备码授权结果（多账号版本，返回账号信息）。
pub async fn copilot_poll_for_account(
    state: &AppState,
    device_code: &str,
    github_domain: Option<&str>,
) -> Result<Option<GitHubAccount>, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.write().await;
    match manager.poll_for_token(device_code, github_domain).await {
        Ok(account) => Ok(account),
        Err(CopilotAuthError::AuthorizationPending) => Ok(None),
        Err(e) => {
            log::error!("[CopilotAuth] 轮询失败: {e}");
            Err(map_auth_error(e))
        }
    }
}

/// 列出所有已认证账号。
pub async fn copilot_list_accounts(state: &AppState) -> Result<Vec<GitHubAccount>, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    Ok(manager.list_accounts().await)
}

/// 移除指定账号。
pub async fn copilot_remove_account(
    state: &AppState,
    account_id: &str,
) -> Result<(), AppError> {
    let manager = copilot_manager(state);
    let manager = manager.write().await;
    manager
        .remove_account(account_id)
        .await
        .map_err(map_auth_error)
}

/// 设置默认账号。
pub async fn copilot_set_default_account(
    state: &AppState,
    account_id: &str,
) -> Result<(), AppError> {
    let manager = copilot_manager(state);
    let manager = manager.write().await;
    manager
        .set_default_account(account_id)
        .await
        .map_err(map_auth_error)
}

/// 获取认证状态。
pub async fn copilot_get_auth_status(state: &AppState) -> Result<CopilotAuthStatus, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    Ok(manager.get_status().await)
}

/// 检查是否已认证。
pub async fn copilot_is_authenticated(state: &AppState) -> Result<bool, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    Ok(manager.is_authenticated().await)
}

/// 注销所有 Copilot 认证。
pub async fn copilot_logout(state: &AppState) -> Result<(), AppError> {
    let manager = copilot_manager(state);
    let manager = manager.write().await;
    manager.clear_auth().await.map_err(map_auth_error)
}

/// 获取有效的 Copilot Token（向后兼容：默认账号）。
pub async fn copilot_get_token(state: &AppState) -> Result<String, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    manager.get_valid_token().await.map_err(map_auth_error)
}

/// 获取指定账号的有效 Token。
pub async fn copilot_get_token_for_account(
    state: &AppState,
    account_id: &str,
) -> Result<String, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    manager
        .get_valid_token_for_account(account_id)
        .await
        .map_err(map_auth_error)
}

/// 获取 Copilot 可用模型列表（默认账号）。
pub async fn copilot_get_models(state: &AppState) -> Result<Vec<CopilotModel>, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    manager.fetch_models().await.map_err(map_auth_error)
}

/// 获取指定账号的可用模型列表。
pub async fn copilot_get_models_for_account(
    state: &AppState,
    account_id: &str,
) -> Result<Vec<CopilotModel>, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    manager
        .fetch_models_for_account(account_id)
        .await
        .map_err(map_auth_error)
}

/// 获取 Copilot 使用量信息（默认账号）。
pub async fn copilot_get_usage(state: &AppState) -> Result<CopilotUsageResponse, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    manager.fetch_usage().await.map_err(map_auth_error)
}

/// 获取指定账号的使用量信息。
pub async fn copilot_get_usage_for_account(
    state: &AppState,
    account_id: &str,
) -> Result<CopilotUsageResponse, AppError> {
    let manager = copilot_manager(state);
    let manager = manager.read().await;
    manager
        .fetch_usage_for_account(account_id)
        .await
        .map_err(map_auth_error)
}

fn map_auth_error(e: CopilotAuthError) -> AppError {
    AppError::Message(e.to_string())
}
