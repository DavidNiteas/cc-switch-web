use crate::error::AppError;
use crate::settings::{get_settings_for_frontend, update_settings, AppSettings};

/// 获取设置（前端展示版本，敏感字段已脱敏）
pub fn get_settings() -> Result<AppSettings, AppError> {
    Ok(get_settings_for_frontend())
}

/// Codex 历史迁移钩子。
///
/// 桌面版注入真实迁移逻辑；无头 Web 版注入空实现，从而避免把依赖大量
/// 本地文件/托盘/重启逻辑的 `codex_history_migration` 模块拖入 core。
pub trait CodexHistoryMigrationHook: Send + Sync {
    /// 用户开启 `unify_codex_session_history` 后调用。
    fn on_unify_codex_enabled(&self) {}
    /// 用户关闭 `unify_codex_session_history` 后调用。
    fn on_unify_codex_disabled(&self) {}
}

/// 无头 Web 版的默认空实现。
pub struct NoOpCodexHistoryMigrationHook;
impl CodexHistoryMigrationHook for NoOpCodexHistoryMigrationHook {}

/// 合并前端传入的设置与后端持久化中的设置。
///
/// 关键规则：
/// - WebDAV / S3 凭据字段若为空，保留后端已有值（因为 `get_settings_for_frontend`
///   会清空密码，空值代表“保持原值”而非“清空”）。
/// - `local_migrations` 是后端维护的迁移标记，前端无权覆盖，始终保留后端值。
pub fn merge_settings_for_save(mut incoming: AppSettings, existing: &AppSettings) -> AppSettings {
    match (&mut incoming.webdav_sync, &existing.webdav_sync) {
        (None, _) => {
            incoming.webdav_sync = existing.webdav_sync.clone();
        }
        (Some(incoming_sync), Some(existing_sync))
            if incoming_sync.password.is_empty() && !existing_sync.password.is_empty() =>
        {
            incoming_sync.password = existing_sync.password.clone();
        }
        _ => {}
    }
    match (&mut incoming.s3_sync, &existing.s3_sync) {
        (None, _) => {
            incoming.s3_sync = existing.s3_sync.clone();
        }
        (Some(incoming_sync), Some(existing_sync))
            if incoming_sync.secret_access_key.is_empty()
                && !existing_sync.secret_access_key.is_empty() =>
        {
            incoming_sync.secret_access_key = existing_sync.secret_access_key.clone();
        }
        _ => {}
    }
    incoming.local_migrations = existing.local_migrations.clone();
    incoming
}

/// 保存设置。
///
/// 当 `unify_codex_session_history` 开关变更时，会立即重写当前官方 Codex
/// 供应商的 live 配置；失败时回滚设置，保持状态一致。
pub fn save_settings(
    app_state: &crate::store::AppState,
    settings: AppSettings,
    hook: &dyn CodexHistoryMigrationHook,
) -> Result<bool, AppError> {
    let existing = crate::settings::get_settings();
    let merged = merge_settings_for_save(settings, &existing);
    let unify_codex_changed =
        merged.unify_codex_session_history != existing.unify_codex_session_history;
    let unify_codex_enabled = merged.unify_codex_session_history;
    update_settings(merged)?;

    if unify_codex_changed {
        if let Err(err) = crate::services::provider::reapply_current_codex_official_live(app_state)
        {
            log::warn!("统一 Codex 会话历史开关变更后重写 live 配置失败，回滚设置: {err}");
            if let Err(rollback_err) = update_settings(existing) {
                log::error!("回滚统一会话开关设置失败: {rollback_err}");
            }
            return Err(AppError::Message(format!(
                "统一 Codex 会话历史开关未生效（live 配置重写失败）: {err}"
            )));
        }

        if unify_codex_enabled {
            hook.on_unify_codex_enabled();
        } else {
            hook.on_unify_codex_disabled();
        }
    }
    Ok(true)
}

// ==============================================================================
// 配置读写命令（A 类，纯 DB 操作）
// ==============================================================================

use crate::store::AppState;

/// 获取当前 app_config_dir 覆盖路径（来自 core 内存缓存）。
pub fn get_app_config_dir_override() -> Option<String> {
    crate::app_store::get_app_config_dir_override().map(|p| p.to_string_lossy().to_string())
}

/// 设置 app_config_dir 覆盖路径（仅写入 core 内存缓存，Web 端不持久化）。
pub fn set_app_config_dir_override(path: Option<&str>) {
    let pathbuf = path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);
    crate::app_store::set_app_config_dir_override(pathbuf);
}

/// 获取整流器配置。
pub fn get_rectifier_config(
    state: &AppState,
) -> Result<crate::proxy::types::RectifierConfig, AppError> {
    state.db.get_rectifier_config()
}

/// 设置整流器配置。
pub fn set_rectifier_config(
    state: &AppState,
    config: crate::proxy::types::RectifierConfig,
) -> Result<bool, AppError> {
    state.db.set_rectifier_config(&config)?;
    Ok(true)
}

/// 获取优化器配置。
pub fn get_optimizer_config(
    state: &AppState,
) -> Result<crate::proxy::types::OptimizerConfig, AppError> {
    state.db.get_optimizer_config()
}

/// 设置优化器配置。
pub fn set_optimizer_config(
    state: &AppState,
    config: crate::proxy::types::OptimizerConfig,
) -> Result<bool, AppError> {
    state.db.set_optimizer_config(&config)?;
    Ok(true)
}

/// 获取 Copilot 优化器配置。
pub fn get_copilot_optimizer_config(
    state: &AppState,
) -> Result<crate::proxy::types::CopilotOptimizerConfig, AppError> {
    state.db.get_copilot_optimizer_config()
}

/// 设置 Copilot 优化器配置。
pub fn set_copilot_optimizer_config(
    state: &AppState,
    config: crate::proxy::types::CopilotOptimizerConfig,
) -> Result<bool, AppError> {
    state.db.set_copilot_optimizer_config(&config)?;
    Ok(true)
}

/// 获取日志配置。
pub fn get_log_config(state: &AppState) -> Result<crate::proxy::types::LogConfig, AppError> {
    state.db.get_log_config()
}

/// 设置日志配置。
pub fn set_log_config(
    state: &AppState,
    config: crate::proxy::types::LogConfig,
) -> Result<bool, AppError> {
    state.db.set_log_config(&config)?;
    log::set_max_level(config.to_level_filter());
    log::info!(
        "日志配置已更新: enabled={}, level={}",
        config.enabled,
        config.level
    );
    Ok(true)
}

// ==============================================================================
// 应用更新检查（D 类降级：原本依赖 Tauri updater，改为直接 HTTP 查询 GitHub releases）
// ==============================================================================

/// Tauri updater 用的 latest.json URL（与 tauri.conf.json `updater.endpoints` 一致）。
const UPDATE_MANIFEST_URL: &str =
    "https://github.com/farion1231/cc-switch/releases/latest/download/latest.json";

/// 应用更新信息。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
}

/// 查询 GitHub releases 是否有新版本。
///
/// 原桌面版用 Tauri updater 检查更新（依赖 `app.updater_builder()`）。
/// Web 模式改为直接 HTTP GET `latest.json`，对比当前 `CARGO_PKG_VERSION`。
///
/// 返回 `Ok(Some(UpdateInfo))` 表示有新版本；`Ok(None)` 表示已是最新；
/// `Err(...)` 表示查询失败（网络错误等）。
pub async fn check_app_update_available() -> Result<Option<UpdateInfo>, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .no_proxy()
        .build()
        .map_err(|e| AppError::Message(format!("初始化更新检查 client 失败: {e}")))?;

    let resp = client
        .get(UPDATE_MANIFEST_URL)
        .send()
        .await
        .map_err(|e| AppError::Message(format!("检查更新失败: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::Message(format!(
            "检查更新失败: HTTP {}",
            resp.status()
        )));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Message(format!("读取更新清单失败: {e}")))?;
    let manifest: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| AppError::Message(format!("解析更新清单失败: {e}")))?;

    let remote_version = manifest
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Message("更新清单缺少 version 字段".to_string()))?
        .to_string();

    // 简单字符串比较：remote != current 即有更新。
    let current_version = env!("CARGO_PKG_VERSION");
    if remote_version == current_version {
        return Ok(None);
    }

    Ok(Some(UpdateInfo {
        version: remote_version,
        notes: manifest
            .get("notes")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        pub_date: manifest
            .get("pub_date")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    }))
}

// ==============================================================================
// 开机自启（B 类，依赖 auto-launch crate，已下沉到 core::auto_launch）
// ==============================================================================

/// 启用/禁用开机自启。
pub fn set_auto_launch(enabled: bool) -> Result<bool, AppError> {
    if enabled {
        crate::auto_launch::enable_auto_launch()?;
    } else {
        crate::auto_launch::disable_auto_launch()?;
    }
    Ok(true)
}

/// 查询当前是否已启用开机自启。
pub fn get_auto_launch_status() -> Result<bool, AppError> {
    crate::auto_launch::is_auto_launch_enabled()
}

// ==============================================================================
// Codex 历史统一会话迁移（B 类，依赖 codex_history_migration，已下沉到 core）
// ==============================================================================

use serde::Serialize;

/// Codex 统一会话还原结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexUnifyHistoryRestoreResult {
    pub restored_jsonl_files: usize,
    pub restored_state_rows: usize,
    /// 还原被跳过的原因（如当前目录没有账本），前端据此提示而非报"成功 0 项"。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_reason: Option<String>,
}

/// 是否存在统一会话开关的迁移备份（决定关闭弹窗里是否显示"恢复备份"勾选）。
pub fn has_codex_unify_history_backup() -> bool {
    crate::codex_history_migration::has_codex_official_history_unify_backup()
}

/// 按迁移备份账本把当时迁入共享桶的官方会话还原回 "openai" 桶。
/// 由关闭统一会话开关的确认弹窗触发；幂等，可安全重试。
///
/// 调用方需在 `spawn_blocking` 上下文执行；这里用同步签名以保持 core 不依赖 tokio。
pub fn restore_codex_unified_history() -> Result<CodexUnifyHistoryRestoreResult, AppError> {
    let outcome = crate::codex_history_migration::restore_codex_official_history_from_backups()?;

    if let Some(reason) = &outcome.skipped_reason {
        log::debug!("○ Codex official history restore skipped: {reason}");
    } else {
        log::info!(
            "✓ Codex official history restored from backups: jsonl_files={}, state_rows={}",
            outcome.restored_jsonl_files,
            outcome.restored_state_rows
        );
    }

    Ok(CodexUnifyHistoryRestoreResult {
        restored_jsonl_files: outcome.restored_jsonl_files,
        restored_state_rows: outcome.restored_state_rows,
        skipped_reason: outcome.skipped_reason,
    })
}
