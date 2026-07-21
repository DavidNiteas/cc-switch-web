use crate::provider::UsageResult;

/// 查询供应商余额。
pub async fn get_balance(base_url: &str, api_key: &str) -> Result<UsageResult, String> {
    crate::services::balance::get_balance(base_url, api_key).await
}
