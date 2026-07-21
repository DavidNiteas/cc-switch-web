//! Usage stats 命令层。
//!
//! 对应 tauri 侧 `commands/usage.rs` 的 11 个命令，封装 `Database` 上的
//! usage_stats 查询方法。`sync_session_usage` 与 `get_usage_data_sources` 见
//! `commands/session_manager.rs`（依赖 session_usage 服务）。

use crate::database::{lock_conn, Database};
use crate::error::AppError;
use crate::services::usage_stats::{
    DailyStats, LogFilters, ModelStats, PaginatedLogs, ProviderLimitStatus, ProviderStats,
    RequestLogDetail, UsageSummary, UsageSummaryByApp,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// 模型定价信息（前端展示用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricingInfo {
    pub model_id: String,
    pub display_name: String,
    pub input_cost_per_million: String,
    pub output_cost_per_million: String,
    pub cache_read_cost_per_million: String,
    pub cache_creation_cost_per_million: String,
}

/// 获取使用量汇总。
pub fn get_usage_summary(
    db: &Database,
    start_date: Option<i64>,
    end_date: Option<i64>,
    app_type: Option<&str>,
    provider_name: Option<&str>,
    model: Option<&str>,
) -> Result<UsageSummary, AppError> {
    db.get_usage_summary(start_date, end_date, app_type, provider_name, model)
}

/// 获取按 app_type 拆分的使用量汇总。
pub fn get_usage_summary_by_app(
    db: &Database,
    start_date: Option<i64>,
    end_date: Option<i64>,
    provider_name: Option<&str>,
    model: Option<&str>,
) -> Result<Vec<UsageSummaryByApp>, AppError> {
    db.get_usage_summary_by_app(start_date, end_date, provider_name, model)
}

/// 获取每日趋势。
pub fn get_usage_trends(
    db: &Database,
    start_date: Option<i64>,
    end_date: Option<i64>,
    app_type: Option<&str>,
    provider_name: Option<&str>,
    model: Option<&str>,
) -> Result<Vec<DailyStats>, AppError> {
    db.get_daily_trends(start_date, end_date, app_type, provider_name, model)
}

/// 获取 Provider 统计。
pub fn get_provider_stats(
    db: &Database,
    start_date: Option<i64>,
    end_date: Option<i64>,
    app_type: Option<&str>,
    provider_name: Option<&str>,
    model: Option<&str>,
) -> Result<Vec<ProviderStats>, AppError> {
    db.get_provider_stats(start_date, end_date, app_type, provider_name, model)
}

/// 获取模型统计。
pub fn get_model_stats(
    db: &Database,
    start_date: Option<i64>,
    end_date: Option<i64>,
    app_type: Option<&str>,
    provider_name: Option<&str>,
    model: Option<&str>,
) -> Result<Vec<ModelStats>, AppError> {
    db.get_model_stats(start_date, end_date, app_type, provider_name, model)
}

/// 获取请求日志列表（分页）。
pub fn get_request_logs(
    db: &Database,
    filters: &LogFilters,
    page: u32,
    page_size: u32,
) -> Result<PaginatedLogs, AppError> {
    db.get_request_logs(filters, page, page_size)
}

/// 获取单个请求详情。
pub fn get_request_detail(
    db: &Database,
    request_id: &str,
) -> Result<Option<RequestLogDetail>, AppError> {
    db.get_request_detail(request_id)
}

/// 获取模型定价列表（前端展示）。
pub fn get_model_pricing(db: &Database) -> Result<Vec<ModelPricingInfo>, AppError> {
    log::info!("获取模型定价列表");
    db.ensure_model_pricing_seeded()?;

    let conn = lock_conn!(db.conn);
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='model_pricing'",
            [],
            |row| row.get::<_, i64>(0).map(|count| count > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        log::error!("model_pricing 表不存在,可能需要重启应用以触发数据库迁移");
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
        "SELECT model_id, display_name, input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
         FROM model_pricing
         ORDER BY display_name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ModelPricingInfo {
            model_id: row.get(0)?,
            display_name: row.get(1)?,
            input_cost_per_million: row.get(2)?,
            output_cost_per_million: row.get(3)?,
            cache_read_cost_per_million: row.get(4)?,
            cache_creation_cost_per_million: row.get(5)?,
        })
    })?;

    let mut pricing = Vec::new();
    for row in rows {
        pricing.push(row?);
    }
    log::info!("成功获取 {} 条模型定价数据", pricing.len());
    Ok(pricing)
}

/// 新增/更新模型定价。
pub fn update_model_pricing(
    db: &Database,
    model_id: &str,
    display_name: &str,
    input_cost: &str,
    output_cost: &str,
    cache_read_cost: &str,
    cache_creation_cost: &str,
) -> Result<(), AppError> {
    let model_id = model_id.trim().to_string();
    let display_name = display_name.trim().to_string();
    if model_id.is_empty() {
        return Err(AppError::localized(
            "usage.modelIdRequired",
            "模型 ID 不能为空",
            "Model ID is required",
        ));
    }
    if display_name.is_empty() {
        return Err(AppError::localized(
            "usage.displayNameRequired",
            "显示名称不能为空",
            "Display name is required",
        ));
    }

    for (label, value) in [
        ("input_cost", input_cost),
        ("output_cost", output_cost),
        ("cache_read_cost", cache_read_cost),
        ("cache_creation_cost", cache_creation_cost),
    ] {
        let parsed = Decimal::from_str(value.trim()).map_err(|e| {
            AppError::localized(
                "usage.invalidPrice",
                format!("{label} 价格无效: {value} - {e}"),
                format!("{label} price is invalid: {value} - {e}"),
            )
        })?;
        if parsed < Decimal::ZERO {
            return Err(AppError::localized(
                "usage.negativePrice",
                format!("{label} 价格不能为负: {value}"),
                format!("{label} price cannot be negative: {value}"),
            ));
        }
    }

    let conn = lock_conn!(db.conn);
    conn.execute(
        "INSERT OR REPLACE INTO model_pricing (
            model_id, display_name, input_cost_per_million, output_cost_per_million,
            cache_read_cost_per_million, cache_creation_cost_per_million
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            model_id,
            display_name,
            input_cost.trim(),
            output_cost.trim(),
            cache_read_cost.trim(),
            cache_creation_cost.trim(),
        ],
    )
    .map_err(|e| AppError::Database(format!("更新模型定价失败: {e}")))?;
    log::info!("已更新模型定价: {model_id}");
    Ok(())
}

/// 检查指定 provider 的使用限额。
pub fn check_provider_limits(
    db: &Database,
    provider_id: &str,
    app_type: &str,
) -> Result<ProviderLimitStatus, AppError> {
    db.check_provider_limits(provider_id, app_type)
}

/// 删除模型定价行。
pub fn delete_model_pricing(db: &Database, model_id: &str) -> Result<(), AppError> {
    let conn = lock_conn!(db.conn);
    conn.execute(
        "DELETE FROM model_pricing WHERE model_id = ?1",
        rusqlite::params![model_id],
    )
    .map_err(|e| AppError::Database(format!("删除模型定价失败: {e}")))?;
    log::info!("已删除模型定价: {model_id}");
    Ok(())
}
