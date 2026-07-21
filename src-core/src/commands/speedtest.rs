use crate::error::AppError;
use crate::services::speedtest::{EndpointLatency, SpeedtestService};

/// 批量测速指定 API 端点。
pub async fn test_api_endpoints(
    urls: Vec<String>,
    timeout_secs: Option<u64>,
) -> Result<Vec<EndpointLatency>, AppError> {
    SpeedtestService::test_endpoints(urls, timeout_secs).await
}
