use crate::app_config::{AppType, McpServer};
use crate::error::AppError;
use crate::services::McpService;
use crate::store::AppState;
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

/// 获取所有 MCP 服务器（统一结构）。
pub fn get_mcp_servers(state: &AppState) -> Result<IndexMap<String, crate::app_config::McpServer>, AppError> {
    McpService::get_all_servers(state)
}

/// 添加或更新 MCP 服务器。
/// `spec` 为 MCP server 的 JSON 配置；`apps` 为启用该 server 的应用列表。
pub fn upsert_mcp_server(
    state: &AppState,
    id: &str,
    spec: Value,
    apps: Vec<String>,
) -> Result<(), AppError> {
    let mut server = if let Some(existing) = state.db.get_all_mcp_servers()?.get(id).cloned() {
        let mut s = existing;
        s.server = spec;
        s
    } else {
        let name = spec
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
        crate::app_config::McpServer {
            id: id.to_string(),
            name,
            server: spec,
            apps: crate::app_config::McpApps::default(),
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        }
    };

    for app in apps {
        let app_type = AppType::from_str(&app)?;
        server.apps.set_enabled_for(&app_type, true);
    }

    McpService::upsert_server(state, server)
}

/// 删除 MCP 服务器。
pub fn delete_mcp_server(state: &AppState, id: &str) -> Result<bool, AppError> {
    McpService::delete_server(state, id)
}

/// 切换某个应用下指定 MCP 服务器的启用状态。
pub fn toggle_mcp_app(
    state: &AppState,
    server_id: &str,
    app: &str,
    enabled: bool,
) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    McpService::toggle_app(state, server_id, app_type, enabled)
}

/// 获取 Claude MCP 状态。
pub fn get_claude_mcp_status() -> Result<crate::claude_mcp::McpStatus, AppError> {
    crate::claude_mcp::get_mcp_status()
}

/// 读取 mcp.json 文本内容。
pub fn read_claude_mcp_config() -> Result<Option<String>, AppError> {
    crate::claude_mcp::read_mcp_json()
}

/// 校验命令是否在 PATH 中可用。
pub fn validate_mcp_command(cmd: &str) -> Result<bool, AppError> {
    crate::claude_mcp::validate_command_in_path(cmd)
}

/// 读取 mcp.json 中的 servers map。
pub fn read_mcp_servers_map() -> Result<std::collections::HashMap<String, serde_json::Value>, AppError> {
    crate::claude_mcp::read_mcp_servers_map()
}

/// 直接操作 Claude mcp.json：新增或更新 server。
pub fn upsert_claude_mcp_server(id: &str, spec: serde_json::Value) -> Result<bool, AppError> {
    crate::claude_mcp::upsert_mcp_server(id, spec)
}

/// 直接操作 Claude mcp.json：删除 server。
pub fn delete_claude_mcp_server(id: &str) -> Result<bool, AppError> {
    crate::claude_mcp::delete_mcp_server(id)
}

/// 兼容层响应：应用配置路径 + 该应用下的 MCP servers map。
#[derive(Serialize)]
pub struct McpConfigResponse {
    pub config_path: String,
    pub servers: HashMap<String, Value>,
}

/// 获取指定应用下的 MCP 服务器（旧分应用 API 兼容层）。
pub fn get_mcp_config(state: &AppState, app: &str) -> Result<McpConfigResponse, AppError> {
    let config_path = crate::config::get_app_config_path()
        .to_string_lossy()
        .to_string();
    let app_ty = AppType::from_str(app)?;
    let servers = McpService::get_servers(state, app_ty)?;
    Ok(McpConfigResponse {
        config_path,
        servers,
    })
}

/// 在 config.json 中新增或更新一个 MCP 服务器定义（旧分应用 API 兼容层）。
pub fn upsert_mcp_server_in_config(
    state: &AppState,
    app: &str,
    id: &str,
    spec: Value,
    sync_other_side: Option<bool>,
) -> Result<bool, AppError> {
    let app_ty = AppType::from_str(app)?;

    let existing_server = state
        .db
        .get_all_mcp_servers()?
        .get(id)
        .cloned();

    let mut new_server = if let Some(mut existing) = existing_server {
        existing.server = spec.clone();
        existing.apps.set_enabled_for(&app_ty, true);
        existing
    } else {
        let mut apps = crate::app_config::McpApps::default();
        apps.set_enabled_for(&app_ty, true);

        let name = spec
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();

        McpServer {
            id: id.to_string(),
            name,
            server: spec,
            apps,
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        }
    };

    if sync_other_side.unwrap_or(false) {
        new_server.apps.claude = true;
        new_server.apps.codex = true;
        new_server.apps.gemini = true;
        new_server.apps.opencode = true;
    }

    McpService::upsert_server(state, new_server)?;
    Ok(true)
}

/// 在 config.json 中删除一个 MCP 服务器定义（旧分应用 API 兼容层）。
pub fn delete_mcp_server_in_config(
    state: &AppState,
    _app: &str,
    id: &str,
) -> Result<bool, AppError> {
    McpService::delete_server(state, id)
}

/// 设置指定应用下 MCP 服务器的启用状态（旧分应用 API 兼容层）。
pub fn set_mcp_enabled(
    state: &AppState,
    app: &str,
    id: &str,
    enabled: bool,
) -> Result<bool, AppError> {
    let app_ty = AppType::from_str(app)?;
    McpService::set_enabled(state, app_ty, id, enabled)?;
    Ok(true)
}

/// 从所有应用导入 MCP 服务器。
pub fn import_mcp_from_apps(state: &AppState) -> Result<usize, AppError> {
    McpService::import_from_all_apps(state)
}
