#![allow(non_snake_case)]

use indexmap::IndexMap;
use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::app_config::AppType;
use crate::app_config::McpServer;
use crate::claude_mcp;
use crate::services::McpService;
use crate::store::AppState;

/// 获取 Claude MCP 状态
#[tauri::command]
pub async fn get_claude_mcp_status() -> Result<claude_mcp::McpStatus, String> {
    cc_switch_core::commands::mcp::get_claude_mcp_status().map_err(|e| e.to_string())
}

/// 读取 mcp.json 文本内容
#[tauri::command]
pub async fn read_claude_mcp_config() -> Result<Option<String>, String> {
    cc_switch_core::commands::mcp::read_claude_mcp_config().map_err(|e| e.to_string())
}

/// 新增或更新一个 MCP 服务器条目
#[tauri::command]
pub async fn upsert_claude_mcp_server(
    id: String,
    spec: Value,
) -> Result<bool, String> {
    cc_switch_core::commands::mcp::upsert_claude_mcp_server(&id, spec).map_err(|e| e.to_string())
}

/// 删除一个 MCP 服务器条目
#[tauri::command]
pub async fn delete_claude_mcp_server(id: String) -> Result<bool, String> {
    cc_switch_core::commands::mcp::delete_claude_mcp_server(&id).map_err(|e| e.to_string())
}

/// 校验命令是否在 PATH 中可用（不执行）
#[tauri::command]
pub async fn validate_mcp_command(cmd: String) -> Result<bool, String> {
    cc_switch_core::commands::mcp::validate_mcp_command(&cmd).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct McpConfigResponse {
    pub config_path: String,
    pub servers: HashMap<String, Value>,
}

/// 获取 MCP 配置（来自 ~/.cc-switch/config.json）
#[tauri::command]
#[allow(deprecated)]
pub async fn get_mcp_config(
    state: State<'_, AppState>,
    app: String,
) -> Result<McpConfigResponse, String> {
    let res = cc_switch_core::commands::mcp::get_mcp_config(&state, &app)
        .map_err(|e| e.to_string())?;
    Ok(McpConfigResponse {
        config_path: res.config_path,
        servers: res.servers,
    })
}

/// 在 config.json 中新增或更新一个 MCP 服务器定义
#[tauri::command]
pub async fn upsert_mcp_server_in_config(
    state: State<'_, AppState>,
    app: String,
    id: String,
    spec: Value,
    sync_other_side: Option<bool>,
) -> Result<bool, String> {
    cc_switch_core::commands::mcp::upsert_mcp_server_in_config(
        &state,
        &app,
        &id,
        spec,
        sync_other_side,
    )
    .map_err(|e| e.to_string())
}

/// 在 config.json 中删除一个 MCP 服务器定义
#[tauri::command]
pub async fn delete_mcp_server_in_config(
    state: State<'_, AppState>,
    _app: String,
    id: String,
) -> Result<bool, String> {
    cc_switch_core::commands::mcp::delete_mcp_server_in_config(&state, &_app, &id)
        .map_err(|e| e.to_string())
}

/// 设置启用状态并同步到客户端配置
#[tauri::command]
#[allow(deprecated)]
pub async fn set_mcp_enabled(
    state: State<'_, AppState>,
    app: String,
    id: String,
    enabled: bool,
) -> Result<bool, String> {
    cc_switch_core::commands::mcp::set_mcp_enabled(&state, &app, &id, enabled)
        .map_err(|e| e.to_string())
}

// ============================================================================
// v3.7.0 新增：统一 MCP 管理命令
// ============================================================================

/// 获取所有 MCP 服务器（统一结构）
#[tauri::command]
pub async fn get_mcp_servers(
    state: State<'_, AppState>,
) -> Result<IndexMap<String, McpServer>, String> {
    McpService::get_all_servers(&state).map_err(|e| e.to_string())
}

/// 添加或更新 MCP 服务器
#[tauri::command]
pub async fn upsert_mcp_server(
    state: State<'_, AppState>,
    server: McpServer,
) -> Result<(), String> {
    McpService::upsert_server(&state, server).map_err(|e| e.to_string())
}

/// 删除 MCP 服务器
#[tauri::command]
pub async fn delete_mcp_server(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    McpService::delete_server(&state, &id).map_err(|e| e.to_string())
}

/// 切换 MCP 服务器在指定应用的启用状态
#[tauri::command]
pub async fn toggle_mcp_app(
    state: State<'_, AppState>,
    server_id: String,
    app: String,
    enabled: bool,
) -> Result<(), String> {
    let app_ty = AppType::from_str(&app).map_err(|e| e.to_string())?;
    McpService::toggle_app(&state, &server_id, app_ty, enabled).map_err(|e| e.to_string())
}

/// 从所有应用导入 MCP 服务器
#[tauri::command]
pub async fn import_mcp_from_apps(state: State<'_, AppState>) -> Result<usize, String> {
    cc_switch_core::commands::mcp::import_mcp_from_apps(&state).map_err(|e| e.to_string())
}
