//! Claude 插件命令（A 类，纯文件读写）。
//!
//! 对应 tauri 侧 `commands/plugin.rs` 的 6 个 A 类命令。
//! 实现已下沉到 `cc_switch_core::claude_plugin`。

use crate::claude_plugin::{
    claude_config_status, clear_claude_config, is_claude_config_applied, read_claude_config,
    write_claude_config,
};
use crate::config::ConfigStatus;
use crate::error::AppError;

/// 获取 `~/.claude/config.json` 状态。
pub fn get_claude_plugin_status() -> Result<ConfigStatus, AppError> {
    let (exists, path) = claude_config_status()?;
    Ok(ConfigStatus {
        exists,
        path: path.to_string_lossy().to_string(),
    })
}

/// 读取 `~/.claude/config.json` 内容（不存在返回 None）。
pub fn read_claude_plugin_config() -> Result<Option<String>, AppError> {
    read_claude_config()
}

/// 写入或清除固定配置。
///
/// - `official = true`：清除 primaryApiKey 字段。
/// - `official = false`：设置 primaryApiKey = "any"。
pub fn apply_claude_plugin_config(official: bool) -> Result<bool, AppError> {
    if official {
        clear_claude_config()
    } else {
        write_claude_config()
    }
}

/// 检测是否已写入 managed 配置（primaryApiKey == "any"）。
pub fn is_claude_plugin_applied() -> Result<bool, AppError> {
    is_claude_config_applied()
}

/// 写入 `~/.claude.json` 的 `hasCompletedOnboarding=true`，跳过初次安装确认。
pub fn apply_claude_onboarding_skip() -> Result<bool, AppError> {
    crate::claude_mcp::set_has_completed_onboarding()
}

/// 清除 `~/.claude.json` 的 `hasCompletedOnboarding` 字段，恢复初次安装确认。
pub fn clear_claude_onboarding_skip() -> Result<bool, AppError> {
    crate::claude_mcp::clear_has_completed_onboarding()
}
