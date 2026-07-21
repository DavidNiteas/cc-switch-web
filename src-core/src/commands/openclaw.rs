//! OpenClaw 应用配置命令（A 类，纯配置文件读写）。
//!
//! 对应 tauri 侧 `commands/openclaw.rs` 中的 14 个 A 类命令。

use std::collections::HashMap;

use crate::error::AppError;
use crate::openclaw_config::{
    get_agents_defaults, get_default_model, get_env_config, get_model_catalog, get_provider,
    get_providers, get_tools_config, set_agents_defaults, set_default_model, set_env_config,
    set_model_catalog, set_tools_config, OpenClawAgentsDefaults, OpenClawDefaultModel,
    OpenClawEnvConfig, OpenClawHealthWarning, OpenClawModelCatalogEntry, OpenClawToolsConfig,
    OpenClawWriteOutcome,
};
use crate::store::AppState;
use serde_json::Value;

/// 从 OpenClaw live 配置导入供应商到数据库。
pub fn import_openclaw_providers_from_live(state: &AppState) -> Result<usize, AppError> {
    crate::services::provider::import_openclaw_providers_from_live(state)
}

/// 获取 OpenClaw live 配置中的供应商 ID 列表。
pub fn get_openclaw_live_provider_ids() -> Result<Vec<String>, AppError> {
    Ok(get_providers()?.keys().cloned().collect())
}

/// 获取指定 OpenClaw 供应商的配置片段。
pub fn get_openclaw_live_provider(provider_id: &str) -> Result<Option<Value>, AppError> {
    get_provider(provider_id)
}

/// 扫描 openclaw.json 中已知配置风险。
pub fn scan_openclaw_config_health() -> Result<Vec<OpenClawHealthWarning>, AppError> {
    crate::openclaw_config::scan_openclaw_config_health()
}

/// 获取 OpenClaw 默认模型配置（agents.defaults.model）。
pub fn get_openclaw_default_model() -> Result<Option<OpenClawDefaultModel>, AppError> {
    get_default_model()
}

/// 设置 OpenClaw 默认模型配置。
pub fn set_openclaw_default_model(
    model: OpenClawDefaultModel,
) -> Result<OpenClawWriteOutcome, AppError> {
    set_default_model(&model)
}

/// 获取 OpenClaw 模型目录/白名单（agents.defaults.models）。
pub fn get_openclaw_model_catalog(
) -> Result<Option<HashMap<String, OpenClawModelCatalogEntry>>, AppError> {
    get_model_catalog()
}

/// 设置 OpenClaw 模型目录/白名单。
pub fn set_openclaw_model_catalog(
    catalog: HashMap<String, OpenClawModelCatalogEntry>,
) -> Result<OpenClawWriteOutcome, AppError> {
    set_model_catalog(&catalog)
}

/// 获取完整 agents.defaults 配置。
pub fn get_openclaw_agents_defaults() -> Result<Option<OpenClawAgentsDefaults>, AppError> {
    get_agents_defaults()
}

/// 设置完整 agents.defaults 配置。
pub fn set_openclaw_agents_defaults(
    defaults: OpenClawAgentsDefaults,
) -> Result<OpenClawWriteOutcome, AppError> {
    set_agents_defaults(&defaults)
}

/// 获取 OpenClaw env 配置节。
pub fn get_openclaw_env() -> Result<OpenClawEnvConfig, AppError> {
    get_env_config()
}

/// 设置 OpenClaw env 配置节。
pub fn set_openclaw_env(env: OpenClawEnvConfig) -> Result<OpenClawWriteOutcome, AppError> {
    set_env_config(&env)
}

/// 获取 OpenClaw tools 配置节。
pub fn get_openclaw_tools() -> Result<OpenClawToolsConfig, AppError> {
    get_tools_config()
}

/// 设置 OpenClaw tools 配置节。
pub fn set_openclaw_tools(tools: OpenClawToolsConfig) -> Result<OpenClawWriteOutcome, AppError> {
    set_tools_config(&tools)
}
