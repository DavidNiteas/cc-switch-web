//! Hermes 应用配置命令（A 类，纯文件/配置读写）。
//!
//! 对应 tauri 侧 `commands/hermes.rs` 中的 8 个 A 类命令。
//! `open_hermes_web_ui` 是 D 类但可降级：Web 模式下返回 URL 给前端，
//! 由前端用 `window.open()` 打开。`launch_hermes_dashboard` 是 D 类
//! （打开系统终端），保留在 tauri 外壳。

use crate::error::AppError;
use crate::hermes_config::{
    get_model_config, get_provider, get_providers, read_memory, read_memory_limits,
    set_memory_enabled, write_memory, HermesMemoryLimits, HermesModelConfig, HermesWriteOutcome,
    MemoryKind,
};
use crate::store::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 从 Hermes live 配置导入供应商到数据库。
pub fn import_hermes_providers_from_live(state: &AppState) -> Result<usize, AppError> {
    crate::services::provider::import_hermes_providers_from_live(state)
}

/// 获取 Hermes live 配置中的供应商 ID 列表。
pub fn get_hermes_live_provider_ids() -> Result<Vec<String>, AppError> {
    Ok(get_providers()?.keys().cloned().collect())
}

/// 获取指定 Hermes 供应商的配置片段。
pub fn get_hermes_live_provider(provider_id: &str) -> Result<Option<Value>, AppError> {
    get_provider(provider_id)
}

/// 获取 Hermes model 配置节（只读）。
pub fn get_hermes_model_config() -> Result<Option<HermesModelConfig>, AppError> {
    get_model_config()
}

/// 读取 Hermes 记忆文件内容。
pub fn get_hermes_memory(kind: MemoryKind) -> Result<String, AppError> {
    read_memory(kind)
}

/// 写入 Hermes 记忆文件内容。
pub fn set_hermes_memory(kind: MemoryKind, content: &str) -> Result<(), AppError> {
    write_memory(kind, content)
}

/// 读取 Hermes 记忆文件的容量限制。
pub fn get_hermes_memory_limits() -> Result<HermesMemoryLimits, AppError> {
    read_memory_limits()
}

/// 启用/禁用 Hermes 记忆文件，并返回写入结果。
pub fn set_hermes_memory_enabled(
    kind: MemoryKind,
    enabled: bool,
) -> Result<HermesWriteOutcome, AppError> {
    set_memory_enabled(kind, enabled)
}

/// Hermes Web UI 探测结果。
///
/// Web 模式下：core 探测 Hermes FastAPI 服务是否在线，返回 URL 字符串。
/// 前端拿到 URL 后用 `window.open(url)` 在浏览器中打开。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesWebUiResult {
    pub url: String,
    pub online: bool,
}

/// 探测 Hermes Web UI 是否在线，并返回完整 URL（D 类降级）。
///
/// 桌面版用 `app.opener().open_url(url)` 在用户浏览器打开；
/// Web 模式下返回 URL 字符串，前端 shim 用 `window.open()` 打开新标签页。
pub async fn open_hermes_web_ui(path: Option<&str>) -> Result<HermesWebUiResult, AppError> {
    use std::time::Duration;

    let port = std::env::var("HERMES_WEB_PORT")
        .ok()
        .and_then(|raw| raw.trim().parse::<u16>().ok())
        .unwrap_or(9119);

    let base = format!("http://127.0.0.1:{port}");
    let probe_url = format!("{base}/api/status");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(1200))
        .no_proxy()
        .build()
        .map_err(|e| AppError::Message(format!("failed to build probe client: {e}")))?;

    let online = match client.get(&probe_url).send().await {
        Ok(_) => true,
        Err(_) => false,
    };

    let target = match path {
        Some(p) if p.starts_with('/') => format!("{base}{p}"),
        Some(p) if !p.is_empty() => format!("{base}/{p}"),
        _ => format!("{base}/"),
    };

    Ok(HermesWebUiResult {
        url: target,
        online,
    })
}
