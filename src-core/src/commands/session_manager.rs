//! Session manager 命令层。
//!
//! 对应 tauri 侧 `commands/session_manager.rs` + `commands/usage.rs` 中的
//! `list_sessions`/`get_session_messages`/`delete_session`/`delete_sessions`/
//! `sync_session_usage`/`get_usage_data_sources`。
//!
//! `launch_session_terminal` 是 D 类（打开系统终端），保留在 tauri 外壳。

use crate::error::AppError;
use crate::services::session_usage::{self, DataSourceSummary, SessionSyncResult};
use crate::session_manager::{self, DeleteSessionOutcome, DeleteSessionRequest, SessionMessage, SessionMeta};
use crate::store::AppState;

/// 扫描所有应用的会话列表。
pub fn list_sessions() -> Vec<SessionMeta> {
    session_manager::scan_sessions()
}

/// 读取指定会话的消息列表。
pub fn get_session_messages(
    provider_id: &str,
    source_path: &str,
) -> Result<Vec<SessionMessage>, AppError> {
    session_manager::load_messages(provider_id, source_path)
        .map_err(|e| AppError::Message(e))
}

/// 删除单个会话（按 provider_id + session_id + source_path 定位）。
pub fn delete_session(
    provider_id: &str,
    session_id: &str,
    source_path: &str,
) -> Result<bool, AppError> {
    session_manager::delete_session(provider_id, session_id, source_path)
        .map_err(|e| AppError::Message(e))
}

/// 批量删除会话。
pub fn delete_sessions(requests: Vec<DeleteSessionRequest>) -> Vec<DeleteSessionOutcome> {
    session_manager::delete_sessions(&requests)
}

/// 同步 Claude/Codex 会话日志到 proxy_request_logs（去重后写入）。
pub fn sync_session_usage(state: &AppState) -> Result<SessionSyncResult, AppError> {
    let mut result = session_usage::sync_claude_session_logs(&state.db)?;
    match crate::services::session_usage_codex::sync_codex_usage(&state.db) {
        Ok(codex_result) => {
            result.imported += codex_result.imported;
            result.skipped += codex_result.skipped;
            result.files_scanned += codex_result.files_scanned;
            if result.errors.is_empty() {
                result.errors = codex_result.errors;
            } else {
                result.errors.extend(codex_result.errors);
            }
        }
        Err(e) => {
            log::warn!("Codex usage sync failed: {e}");
            result.errors.push(format!("Codex: {e}"));
        }
    }
    Ok(result)
}

/// 获取使用量数据源（proxy/session_log 等）的分布统计。
pub fn get_usage_data_sources(state: &AppState) -> Result<Vec<DataSourceSummary>, AppError> {
    session_usage::get_data_source_breakdown(&state.db)
}
