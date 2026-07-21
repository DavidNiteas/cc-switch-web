use crate::error::AppError;
use crate::services::subscription::SubscriptionQuota;

/// 查询官方订阅额度。
///
/// 注意：桌面版会同时写入 UsageCache、发射 `usage-cache-updated` 事件并刷新托盘。
/// core 命令只返回原始结果，事件/缓存/托盘逻辑由 tauri/web 壳处理。
pub async fn get_subscription_quota(tool: &str) -> Result<SubscriptionQuota, AppError> {
    crate::services::subscription::get_subscription_quota(tool)
        .await
        .map_err(|e| AppError::Message(e))
}
