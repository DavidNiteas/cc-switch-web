//! Coding plan quota 命令层。
//!
//! 对应 tauri 侧 `commands/coding_plan.rs`，转发到
//! `services::coding_plan::get_coding_plan_quota`。

use crate::error::AppError;
use crate::services::subscription::SubscriptionQuota;

/// 查询 Coding Plan 额度（Kimi/智谱/MiniMax/ZenMux/火山等）。
#[allow(clippy::too_many_arguments)]
pub async fn get_coding_plan_quota(
    base_url: &str,
    api_key: &str,
    access_key_id: Option<&str>,
    secret_access_key: Option<&str>,
    coding_plan_provider: Option<&str>,
    team_organization_id: Option<&str>,
    team_project_id: Option<&str>,
) -> Result<SubscriptionQuota, AppError> {
    crate::services::coding_plan::get_coding_plan_quota(
        base_url,
        api_key,
        access_key_id,
        secret_access_key,
        coding_plan_provider,
        team_organization_id,
        team_project_id,
    )
    .await
    .map_err(|e| AppError::Message(e.to_string()))
}
