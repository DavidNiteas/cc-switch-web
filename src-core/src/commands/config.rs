use crate::app_config::AppType;
use crate::config::{self, ConfigStatus};
use crate::database::Database;
use crate::error::AppError;
use std::str::FromStr;

/// 获取指定应用的配置状态。
pub fn get_config_status(db: &Database, app: &str, proxy_running: bool) -> Result<ConfigStatus, AppError> {
    let app_type = AppType::from_str(app)?;
    match app_type {
        AppType::Claude => Ok(crate::config::get_claude_config_status()),
        AppType::ClaudeDesktop => {
            let status = crate::claude_desktop_config::get_status(db, proxy_running)?;
            Ok(ConfigStatus {
                exists: status.configured,
                path: status.config_library_path.unwrap_or_default(),
            })
        }
        AppType::Codex => {
            let auth_path = crate::codex_config::get_codex_auth_path();
            let config_text = crate::codex_config::read_codex_config_text().unwrap_or_default();
            let exists = auth_path.exists() || !config_text.trim().is_empty();
            let path = crate::codex_config::get_codex_config_dir()
                .to_string_lossy()
                .to_string();
            Ok(ConfigStatus { exists, path })
        }
        AppType::Gemini => {
            let env_path = crate::gemini_config::get_gemini_env_path();
            let exists = env_path.exists();
            let path = crate::gemini_config::get_gemini_dir()
                .to_string_lossy()
                .to_string();
            Ok(ConfigStatus { exists, path })
        }
        AppType::GrokBuild => {
            let config_path = crate::grok_config::get_grok_config_path();
            let exists = config_path.exists();
            let path = crate::grok_config::get_grok_config_dir()
                .to_string_lossy()
                .to_string();
            Ok(ConfigStatus { exists, path })
        }
        AppType::OpenCode => {
            let config_path = crate::opencode_config::get_opencode_config_path();
            let exists = config_path.exists();
            let path = crate::opencode_config::get_opencode_dir()
                .to_string_lossy()
                .to_string();
            Ok(ConfigStatus { exists, path })
        }
        AppType::OpenClaw => {
            let config_path = crate::openclaw_config::get_openclaw_config_path();
            let exists = config_path.exists();
            let path = crate::openclaw_config::get_openclaw_dir()
                .to_string_lossy()
                .to_string();
            Ok(ConfigStatus { exists, path })
        }
        AppType::Hermes => {
            let config_path = crate::hermes_config::get_hermes_config_path();
            let exists = config_path.exists();
            let path = config_path.to_string_lossy().to_string();
            Ok(ConfigStatus { exists, path })
        }
    }
}

/// 获取指定应用的配置目录路径。
pub fn get_config_dir(app: &str) -> Result<String, AppError> {
    let app_type = AppType::from_str(app)?;
    let dir = match app_type {
        AppType::Claude => config::get_claude_config_dir(),
        AppType::ClaudeDesktop => {
            crate::claude_desktop_config::get_config_library_path()?
        }
        AppType::Codex => crate::codex_config::get_codex_config_dir(),
        AppType::Gemini => crate::gemini_config::get_gemini_dir(),
        AppType::GrokBuild => crate::grok_config::get_grok_config_dir(),
        AppType::OpenCode => crate::opencode_config::get_opencode_dir(),
        AppType::OpenClaw => crate::openclaw_config::get_openclaw_dir(),
        AppType::Hermes => crate::hermes_config::get_hermes_dir(),
    };
    Ok(dir.to_string_lossy().to_string())
}

/// 获取 Claude Code 配置文件路径。
pub fn get_claude_code_config_path() -> Result<String, AppError> {
    Ok(config::get_claude_settings_path().to_string_lossy().to_string())
}

/// 获取 cc-switch 应用配置目录路径。
pub fn get_app_config_path() -> Result<String, AppError> {
    Ok(config::get_app_config_path().to_string_lossy().to_string())
}

/// 获取 common config snippet（如果已设置）。
pub fn get_config_snippet(db: &Database, app: &str) -> Result<Option<String>, AppError> {
    let app_type = AppType::from_str(app)?;
    db.get_config_snippet(app_type.as_str())
}

/// 设置 common config snippet。
pub fn set_config_snippet(db: &Database, app: &str, snippet: Option<String>) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    db.set_config_snippet(app_type.as_str(), snippet)
}

/// 清除 common config snippet 标记。
pub fn clear_config_snippet(db: &Database, app: &str) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    db.set_config_snippet_cleared(app_type.as_str(), true)
}

/// 获取 Claude 独立配置状态。
pub fn get_claude_config_status() -> Result<ConfigStatus, AppError> {
    Ok(crate::config::get_claude_config_status())
}

fn invalid_json_format_error(error: serde_json::Error) -> String {
    let lang = crate::settings::get_settings()
        .language
        .unwrap_or_else(|| "zh".to_string());

    match lang.as_str() {
        "en" => format!("Invalid JSON format: {error}"),
        "ja" => format!("JSON形式が無効です: {error}"),
        _ => format!("无效的 JSON 格式: {error}"),
    }
}

fn invalid_toml_format_error(error: toml_edit::TomlError) -> String {
    let lang = crate::settings::get_settings()
        .language
        .unwrap_or_else(|| "zh".to_string());

    match lang.as_str() {
        "en" => format!("Invalid TOML format: {error}"),
        "ja" => format!("TOML形式が無効です: {error}"),
        _ => format!("无效的 TOML 格式: {error}"),
    }
}

fn validate_common_config_snippet(app_type: &str, snippet: &str) -> Result<(), String> {
    if snippet.trim().is_empty() {
        return Ok(());
    }

    match app_type {
        "claude" | "gemini" | "omo" | "omo-slim" => {
            serde_json::from_str::<serde_json::Value>(snippet)
                .map_err(invalid_json_format_error)?;
        }
        "codex" => {
            snippet
                .parse::<toml_edit::DocumentMut>()
                .map_err(invalid_toml_format_error)?;
        }
        _ => {}
    }

    Ok(())
}

/// 获取 Claude 通用配置片段。
pub fn get_claude_common_config_snippet(db: &Database) -> Result<Option<String>, AppError> {
    db.get_config_snippet("claude")
}

/// 设置 Claude 通用配置片段。
pub fn set_claude_common_config_snippet(
    db: &Database,
    snippet: &str,
) -> Result<(), AppError> {
    let is_cleared = snippet.trim().is_empty();

    if !is_cleared {
        serde_json::from_str::<serde_json::Value>(snippet)
            .map_err(|e| AppError::Message(invalid_json_format_error(e)))?;
    }

    let value = if is_cleared { None } else { Some(snippet.to_string()) };

    db.set_config_snippet("claude", value)?;
    db.set_config_snippet_cleared("claude", is_cleared)?;
    Ok(())
}

/// 获取指定应用的通用配置片段。
pub fn get_common_config_snippet(
    db: &Database,
    app_type: &str,
) -> Result<Option<String>, AppError> {
    let app_type = AppType::from_str(app_type)?;
    db.get_config_snippet(app_type.as_str())
}

/// 对前端编辑器里的 Codex config.toml 文本做通用配置片段的合并/剥离。
pub fn update_toml_common_config_snippet(
    config_toml: &str,
    snippet_toml: &str,
    enabled: bool,
) -> Result<String, AppError> {
    crate::services::provider::update_toml_common_config_snippet(
        config_toml,
        snippet_toml,
        enabled,
    )
}

// ============================================================================
// P4-B：opener 文件夹类命令（D 类降级：返回路径给前端展示）
// ============================================================================

use serde::{Deserialize, Serialize};

/// opener 文件夹命令的返回结构。
///
/// 桌面版用 `app.opener().open_path(dir)` 在用户桌面打开文件夹。
/// Web 模式下无法打开用户本地文件夹，返回服务器上的路径字符串 +
/// 提示信息，前端展示路径并提供"复制到剪贴板"按钮。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderInfo {
    pub path: String,
    pub exists: bool,
    pub message: String,
}

/// 返回 Claude 应用配置目录路径（D 类降级：原 `open_app_config_folder`）。
pub fn open_app_config_folder() -> Result<FolderInfo, AppError> {
    let path_str = get_app_config_path()?;
    let parent = std::path::Path::new(&path_str)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from(&path_str));
    let exists = parent.exists();
    if !exists {
        std::fs::create_dir_all(&parent)
            .map_err(|e| AppError::Message(format!("创建目录失败: {e}")))?;
    }
    Ok(FolderInfo {
        path: parent.to_string_lossy().to_string(),
        exists: true,
        message: "Server path. Cannot open in your local file manager from Web mode.".to_string(),
    })
}

/// 返回指定应用的配置目录路径（D 类降级：原 `open_config_folder`）。
pub fn open_config_folder(app: &str) -> Result<FolderInfo, AppError> {
    let dir = get_config_dir(app)?;
    let path = std::path::PathBuf::from(&dir);
    let exists = path.exists();
    if !exists {
        std::fs::create_dir_all(&path)
            .map_err(|e| AppError::Message(format!("创建目录失败: {e}")))?;
    }
    Ok(FolderInfo {
        path: dir,
        exists: true,
        message: "Server path. Cannot open in your local file manager from Web mode.".to_string(),
    })
}

/// 返回 OpenClaw workspace 目录路径（D 类降级：原 `open_workspace_directory`）。
pub fn open_workspace_directory(subdir: &str) -> Result<FolderInfo, AppError> {
    let dir = match subdir {
        "memory" => crate::openclaw_config::get_openclaw_dir().join("workspace").join("memory"),
        _ => crate::openclaw_config::get_openclaw_dir().join("workspace"),
    };
    let exists = dir.exists();
    if !exists {
        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::Message(format!("Failed to create directory: {e}")))?;
    }
    Ok(FolderInfo {
        path: dir.to_string_lossy().to_string(),
        exists: true,
        message: "Server path. Cannot open in your local file manager from Web mode.".to_string(),
    })
}

/// 设置指定应用的通用配置片段。
pub fn set_common_config_snippet(
    state: &crate::store::AppState,
    app_type: &str,
    snippet: &str,
) -> Result<(), AppError> {
    use crate::services::omo::{OmoService, SLIM, STANDARD};
    use crate::services::provider::ProviderService;

    let is_cleared = snippet.trim().is_empty();
    let old_snippet = state.db.get_config_snippet(app_type)?;

    validate_common_config_snippet(app_type, snippet)
        .map_err(AppError::Message)?;

    let value = if is_cleared { None } else { Some(snippet.to_string()) };

    if matches!(app_type, "claude" | "codex" | "gemini") {
        if let Some(legacy_snippet) = old_snippet
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let app = AppType::from_str(app_type)?;
            ProviderService::migrate_legacy_common_config_usage(
                state,
                app,
                legacy_snippet,
            )?;
        }
    }

    state.db.set_config_snippet(app_type, value)?;
    state.db.set_config_snippet_cleared(app_type, is_cleared)?;

    if matches!(app_type, "claude" | "codex" | "gemini") {
        let app = AppType::from_str(app_type)?;
        ProviderService::sync_current_provider_for_app(state, app)?;
    }

    if app_type == "omo"
        && state
            .db
            .get_current_omo_provider("opencode", "omo")?
            .is_some()
    {
        OmoService::write_config_to_file(state, &STANDARD)?;
    }
    if app_type == "omo-slim"
        && state
            .db
            .get_current_omo_provider("opencode", "omo-slim")?
            .is_some()
    {
        OmoService::write_config_to_file(state, &SLIM)?;
    }
    Ok(())
}

/// 从当前供应商或指定配置中提取通用配置片段。
pub fn extract_common_config_snippet(
    state: &crate::store::AppState,
    app_type: &str,
    settings_config: Option<&str>,
) -> Result<String, AppError> {
    use crate::services::provider::ProviderService;

    let app = AppType::from_str(app_type)?;

    if let Some(settings_config) = settings_config.filter(|s| !s.trim().is_empty()) {
        let settings: serde_json::Value =
            serde_json::from_str(settings_config)
                .map_err(|e| AppError::Message(invalid_json_format_error(e)))?;
        return ProviderService::extract_common_config_snippet_from_settings(
            app,
            &settings,
        );
    }

    ProviderService::extract_common_config_snippet(state, app)
}

#[cfg(test)]
mod tests {
    use super::validate_common_config_snippet;

    #[test]
    fn validate_common_config_snippet_accepts_comment_only_codex_snippet() {
        validate_common_config_snippet("codex", "# comment only\n")
            .expect("comment-only codex snippet should be valid");
    }

    #[test]
    fn validate_common_config_snippet_rejects_invalid_codex_snippet() {
        let err = validate_common_config_snippet("codex", "[broken")
            .expect_err("invalid codex snippet should be rejected");
        assert!(
            err.contains("TOML") || err.contains("toml") || err.contains("格式"),
            "expected TOML validation error, got {err}"
        );
    }
}
