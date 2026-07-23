use crate::database::{Database, FailoverQueueItem};
use crate::error::AppError;
use crate::provider::Provider;
use std::str::FromStr;

/// 获取故障转移队列。
pub fn get_failover_queue(
    db: &Database,
    app_type: &str,
) -> Result<Vec<FailoverQueueItem>, AppError> {
    db.get_failover_queue(app_type)
}

/// 获取可添加到故障转移队列的供应商。
pub fn get_available_providers_for_failover(
    db: &Database,
    app_type: &str,
) -> Result<Vec<Provider>, AppError> {
    db.get_available_providers_for_failover(app_type)
}

/// 添加供应商到故障转移队列。
pub fn add_to_failover_queue(
    db: &Database,
    app_type: &str,
    provider_id: &str,
) -> Result<(), AppError> {
    db.add_to_failover_queue(app_type, provider_id)
}

/// 从故障转移队列移除供应商。
pub fn remove_from_failover_queue(
    db: &Database,
    app_type: &str,
    provider_id: &str,
) -> Result<(), AppError> {
    db.remove_from_failover_queue(app_type, provider_id)
}

/// 获取指定应用的自动故障转移开关状态。
pub async fn get_auto_failover_enabled(db: &Database, app_type: &str) -> Result<bool, AppError> {
    let config = db.get_proxy_config_for_app(app_type).await?;
    Ok(config.auto_failover_enabled)
}

/// 设置自动故障转移开关后的结果。
#[derive(Debug, Clone)]
pub struct SetAutoFailoverResult {
    pub enabled: bool,
    /// 开启故障转移时，自动切换到的 P1 供应商 ID。
    pub p1_provider_id: Option<String>,
}

/// 设置指定应用的自动故障转移开关状态。
///
/// 桌面版在调用后应自行发射 `provider-switched` 事件并刷新托盘菜单；
/// 无头 Web 版直接忽略这些 UI 副作用。
pub async fn set_auto_failover_enabled(
    app_state: &crate::store::AppState,
    app_type: &str,
    enabled: bool,
) -> Result<SetAutoFailoverResult, AppError> {
    log::info!(
        "[Failover] Setting auto_failover_enabled: app_type='{app_type}', enabled={enabled}"
    );

    let mut config = app_state.db.get_proxy_config_for_app(app_type).await?;

    if enabled && !config.enabled {
        return Err(AppError::Message(
            "需要先启用该应用的代理接管，再开启故障转移".to_string(),
        ));
    }

    let mut auto_added_provider_id: Option<String> = None;
    let p1_provider_id = if enabled {
        let mut queue = app_state.db.get_failover_queue(app_type)?;

        if queue.is_empty() {
            let app_enum = crate::app_config::AppType::from_str(app_type)
                .map_err(|_| AppError::Message(format!("无效的应用类型: {app_type}")))?;

            let current_id =
                crate::settings::get_effective_current_provider(&app_state.db, &app_enum)?;

            let Some(current_id) = current_id else {
                return Err(AppError::Message(
                    "故障转移队列为空，且未设置当前供应商，无法开启故障转移".to_string(),
                ));
            };

            app_state.db.add_to_failover_queue(app_type, &current_id)?;
            auto_added_provider_id = Some(current_id);

            queue = app_state.db.get_failover_queue(app_type)?;
        }

        queue
            .first()
            .map(|item| item.provider_id.clone())
            .ok_or_else(|| AppError::Message("故障转移队列为空，无法开启故障转移".to_string()))?
    } else {
        String::new()
    };

    if enabled {
        if let Err(e) = app_state
            .proxy_service
            .switch_proxy_target(app_type, &p1_provider_id)
            .await
        {
            if let Some(provider_id) = auto_added_provider_id {
                let _ = app_state
                    .db
                    .remove_from_failover_queue(app_type, &provider_id);
            }
            return Err(AppError::Message(e));
        }
    }

    config.auto_failover_enabled = enabled;
    app_state.db.update_proxy_config_for_app(config).await?;

    Ok(SetAutoFailoverResult {
        enabled,
        p1_provider_id: if enabled { Some(p1_provider_id) } else { None },
    })
}
