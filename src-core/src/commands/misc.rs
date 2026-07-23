use crate::error::AppError;
use crate::init_status::{
    get_init_error, take_migration_success, take_skills_migration_result, InitErrorPayload,
    SkillsMigrationPayload,
};
use crate::platform::Platform;

/// 判断是否为便携版（绿色版）运行
pub fn is_portable_mode() -> Result<bool, AppError> {
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::Message(format!("获取可执行路径失败: {e}")))?;
    if let Some(dir) = exe_path.parent() {
        Ok(dir.join("portable.ini").is_file())
    } else {
        Ok(false)
    }
}

/// 获取应用启动阶段的初始化错误（若有）
pub fn get_init_error_command() -> Result<Option<InitErrorPayload>, AppError> {
    Ok(get_init_error())
}

/// 获取 JSON→SQLite 迁移结果（只返回一次 true）。
pub fn get_migration_result() -> Result<bool, AppError> {
    Ok(take_migration_success())
}

/// 获取 Skills 自动导入（SSOT）迁移结果（只返回一次）。
pub fn get_skills_migration_result() -> Result<Option<SkillsMigrationPayload>, AppError> {
    Ok(take_skills_migration_result())
}

/// 检查更新：在系统浏览器打开 release 页面。
pub async fn check_for_updates(platform: &dyn Platform) -> Result<bool, AppError> {
    platform
        .open_url("https://github.com/farion1231/cc-switch/releases/latest")
        .await
        .map_err(|e| AppError::Message(format!("打开更新页面失败: {e}")))?;
    Ok(true)
}

/// 写入系统剪贴板
pub async fn copy_text_to_clipboard(
    _platform: &dyn Platform,
    text: String,
) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| AppError::Message(format!("访问系统剪贴板失败: {e}")))?;
        clipboard
            .set_text(text)
            .map_err(|e| AppError::Message(format!("写入系统剪贴板失败: {e}")))?;
        Ok(true)
    })
    .await
    .map_err(|e| AppError::Message(format!("剪贴板任务执行失败: {e}")))?
}

/// 在系统浏览器中打开外部链接
pub async fn open_external(platform: &dyn Platform, url: String) -> Result<bool, AppError> {
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url
    } else {
        format!("https://{url}")
    };

    platform
        .open_url(&url)
        .await
        .map_err(|e| AppError::Message(format!("打开链接失败: {e}")))?;

    Ok(true)
}

// ============================================================================
// set_window_theme（D 类降级）
// ============================================================================

/// 设置窗口主题。
///
/// 桌面版用 `window.set_theme(Theme::Dark/Light)` 让 Tauri 窗口边框和 WebView
/// 主题一致。Web 模式下 CSS 主题由前端 `prefers-color-scheme` 媒体查询 +
/// localStorage 控制，此命令静默成功（no-op），不影响前端 UI 状态。
///
/// 参数：`"dark"` / `"light"` / `"system"`。
pub fn set_window_theme(_theme: &str) -> Result<(), AppError> {
    log::debug!("[set_window_theme] Web 模式下为 no-op，theme={}", _theme);
    Ok(())
}

// ============================================================================
// P4-B：系统终端命令（D 类降级：返回命令字符串给前端展示）
// ============================================================================

/// 终端启动命令的返回结构。
///
/// 桌面版在用户桌面打开终端运行命令；Web 模式无法打开终端，返回命令 +
/// cwd + env_vars 字符串，前端展示并允许用户"复制到剪贴板"在本地终端运行。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalLaunchInfo {
    pub command: String,
    pub cwd: Option<String>,
    pub env_vars: Vec<(String, String)>,
    pub message: String,
}

/// 解析 provider 配置生成终端命令（D 类降级：原 `open_provider_terminal`）。
///
/// 桌面版：在用户桌面打开终端，注入 env_vars，cd 到 cwd。
/// Web 模式：返回命令结构，前端展示并允许用户手动复制到本地终端运行。
pub async fn open_provider_terminal(
    state: &crate::store::AppState,
    app: &str,
    provider_id: &str,
    cwd: Option<&str>,
) -> Result<TerminalLaunchInfo, AppError> {
    use crate::app_config::AppType;
    use std::str::FromStr;

    let app_type = AppType::from_str(app)?;
    let providers = crate::services::ProviderService::list(state, app_type.clone())
        .map_err(|e| AppError::Message(format!("获取提供商列表失败: {e}")))?;
    let provider = providers
        .get(provider_id)
        .ok_or_else(|| AppError::Message(format!("提供商 {provider_id} 不存在")))?;

    let env_vars = extract_env_vars_from_config(&provider.settings_config, &app_type);

    // 生成展示命令：export KEY=VALUE; <shell>
    let mut export_lines = Vec::new();
    for (k, v) in &env_vars {
        export_lines.push(format!("export {k}={v:?}"));
    }
    let command = if export_lines.is_empty() {
        "$SHELL".to_string()
    } else {
        format!("{}; exec $SHELL", export_lines.join(" && "))
    };

    Ok(TerminalLaunchInfo {
        command,
        cwd: cwd.map(|s| s.to_string()),
        env_vars,
        message: "Web mode: cannot open terminal. Copy the command to run in your local shell."
            .to_string(),
    })
}

/// 返回 session terminal 启动命令字符串（D 类降级：原 `launch_session_terminal`）。
pub fn launch_session_terminal(
    command: &str,
    cwd: Option<&str>,
) -> Result<TerminalLaunchInfo, AppError> {
    Ok(TerminalLaunchInfo {
        command: command.to_string(),
        cwd: cwd.map(|s| s.to_string()),
        env_vars: vec![],
        message: "Web mode: cannot open terminal. Copy the command to run in your local shell."
            .to_string(),
    })
}

/// 从提供商配置中提取环境变量（与桌面版同名 helper 行为一致）。
fn extract_env_vars_from_config(
    config: &serde_json::Value,
    app_type: &crate::app_config::AppType,
) -> Vec<(String, String)> {
    use crate::app_config::AppType;
    let mut env_vars = Vec::new();

    let Some(obj) = config.as_object() else {
        return env_vars;
    };

    if let Some(env) = obj.get("env").and_then(|v| v.as_object()) {
        for (key, value) in env {
            if let Some(str_val) = value.as_str() {
                env_vars.push((key.clone(), str_val.to_string()));
            }
        }
        let base_url_key = match app_type {
            AppType::Claude | AppType::ClaudeDesktop => Some("ANTHROPIC_BASE_URL"),
            AppType::Gemini => Some("GOOGLE_GEMINI_BASE_URL"),
            _ => None,
        };
        if let Some(key) = base_url_key {
            if let Some(url_str) = env.get(key).and_then(|v| v.as_str()) {
                env_vars.push((key.to_string(), url_str.to_string()));
            }
        }
    }

    if *app_type == AppType::Codex {
        if let Some(auth) = obj.get("auth").and_then(|v| v.as_str()) {
            env_vars.push(("OPENAI_API_KEY".to_string(), auth.to_string()));
        }
    }
    if *app_type == AppType::Gemini {
        if let Some(api_key) = obj.get("api_key").and_then(|v| v.as_str()) {
            env_vars.push(("GEMINI_API_KEY".to_string(), api_key.to_string()));
        }
    }

    env_vars
}

// ============================================================================
// 工具版本/安装探测（E 类，简化版实现）
// ============================================================================

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Tauri 命令签名约定的 7 个支持工具。
const VALID_TOOLS: [&str; 7] = [
    "claude", "codex", "gemini", "grok", "opencode", "openclaw", "hermes",
];

/// 工具版本探测结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolVersion {
    pub name: String,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub error: Option<String>,
    pub installed_but_broken: bool,
    pub env_type: String,
    pub wsl_distro: Option<String>,
}

/// 探测指定工具集的本地版本。
///
/// **简化版**：循环跑 `tool --version`，解析输出。
/// 桌面版有更复杂的逻辑（WSL 路由、多版本枚举、latest 查询等），Web 模式暂只实现
/// 基础探测，足以告诉前端"装了没、什么版本"。
pub async fn get_tool_versions(tools: Option<Vec<String>>) -> Result<Vec<ToolVersion>, AppError> {
    let requested: Vec<&str> = if let Some(tools) = tools.as_ref() {
        let set: std::collections::HashSet<&str> = tools.iter().map(|s| s.as_str()).collect();
        VALID_TOOLS
            .iter()
            .copied()
            .filter(|t| set.contains(t))
            .collect()
    } else {
        VALID_TOOLS.to_vec()
    };

    let env_type = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let mut results = Vec::new();
    for tool in requested {
        let mut version = None;
        let mut error = None;
        let mut installed_but_broken = false;

        match Command::new(tool).arg("--version").output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                version = parse_version_from_output(&stdout);
                if version.is_none() {
                    version = Some(stdout.trim().to_string());
                }
            }
            Ok(output) => {
                installed_but_broken = true;
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                error = Some(format!("exit {}: {}", output.status, stderr.trim()));
            }
            Err(e) => {
                error = Some(format!("未安装: {e}"));
            }
        }

        results.push(ToolVersion {
            name: tool.to_string(),
            version,
            latest_version: None, // 桌面版会查 npm registry，Web 简化版暂不实现
            error,
            installed_but_broken,
            env_type: env_type.to_string(),
            wsl_distro: None,
        });
    }
    Ok(results)
}

/// 从 `tool --version` 的输出中提取版本号。
fn parse_version_from_output(stdout: &str) -> Option<String> {
    // 常见格式：`claude 1.2.3` / `v1.2.3` / `1.2.3` / `tool (CLI) v1.2.3`
    for line in stdout.lines().take(3) {
        // 找形如 1.2.3 / 1.2.3-beta 的版本号
        let tokens: Vec<&str> = line.split_whitespace().collect();
        for token in &tokens {
            if let Some(v) = extract_version(token) {
                return Some(v);
            }
        }
    }
    None
}

/// 从单个 token 提取版本号（支持 `1.2.3` / `v1.2.3` / `(1.2.3)` 等）。
fn extract_version(token: &str) -> Option<String> {
    let cleaned = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-');
    // 必须包含至少一个点号且首字符是数字
    if cleaned.contains('.')
        && cleaned
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        return Some(cleaned.to_string());
    }
    if let Some(rest) = cleaned.strip_prefix('v') {
        if rest.contains('.')
            && rest
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some(rest.to_string());
        }
    }
    None
}

/// 工具安装分布报告（简化版）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInstallation {
    pub path: String,
    pub version: Option<String>,
    pub runnable: bool,
    pub error: Option<String>,
    pub source: String,
    pub is_path_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInstallationReport {
    pub tool: String,
    pub installs: Vec<ToolInstallation>,
    pub is_conflict: bool,
    pub needs_confirmation: bool,
    pub command: String,
    pub anchored: bool,
}

/// 探测指定工具集的安装分布。
///
/// **简化版**：只查 `which tool` 找到 PATH 中的默认入口，跑 `--version` 验证。
/// 桌面版会枚举 nvm/homebrew/npm root 等多路径并做冲突诊断，Web 模式暂只实现
/// 基础探测。
pub async fn probe_tool_installations(
    tools: Vec<String>,
) -> Result<Vec<ToolInstallationReport>, AppError> {
    let requested: Vec<&str> = VALID_TOOLS
        .iter()
        .copied()
        .filter(|t| tools.iter().any(|s| s.as_str() == *t))
        .collect();
    if requested.is_empty() {
        return Err(AppError::Message("No supported tools selected".to_string()));
    }

    let mut reports = Vec::new();
    for tool in requested {
        let path = which_tool(tool);
        let (version, runnable, error) = match &path {
            Some(p) => match Command::new(p).arg("--version").output() {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    (parse_version_from_output(&stdout), true, None)
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    (
                        None,
                        false,
                        Some(format!("exit {}: {}", out.status, stderr.trim())),
                    )
                }
                Err(e) => (None, false, Some(e.to_string())),
            },
            None => (None, false, Some("not in PATH".to_string())),
        };

        let install = ToolInstallation {
            path: path.clone().unwrap_or_default(),
            version,
            runnable,
            error,
            source: infer_source(&path),
            is_path_default: path.is_some(),
        };

        let command = upgrade_command_for(tool);
        reports.push(ToolInstallationReport {
            tool: tool.to_string(),
            installs: vec![install.clone()],
            is_conflict: false, // 单个安装不会冲突
            needs_confirmation: false,
            command,
            anchored: path.is_some(),
        });
    }
    Ok(reports)
}

/// `which tool` 的跨平台实现。
fn which_tool(tool: &str) -> Option<String> {
    let cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    let output = Command::new(cmd).arg(tool).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let first_line = stdout.lines().next()?;
    if first_line.is_empty() {
        None
    } else {
        Some(first_line.to_string())
    }
}

/// 由路径前缀推断安装来源。
fn infer_source(path: &Option<String>) -> String {
    let Some(p) = path else {
        return "unknown".to_string();
    };
    let lower = p.to_ascii_lowercase();
    if lower.contains("/.nvm/") || lower.contains("\\nvm\\") {
        "nvm".to_string()
    } else if lower.contains("/homebrew/") || lower.contains("/cellar/") {
        "homebrew".to_string()
    } else if lower.contains("/.npm-global/") || lower.contains("/lib/node_modules/") {
        "npm-global".to_string()
    } else if lower.contains("/.local/") {
        "local".to_string()
    } else {
        "path".to_string()
    }
}

/// 工具升级命令（简化版：统一用 npm update -g）。
fn upgrade_command_for(tool: &str) -> String {
    let pkg = match tool {
        "claude" => "@anthropic-ai/claude-code",
        "codex" => "@openai/codex",
        "gemini" => "@google/gemini-cli",
        "grok" => "@xai/grok-cli",
        "opencode" => "opencode-ai",
        "openclaw" => "@openclaw/cli",
        "hermes" => "@hermes-agent/cli",
        _ => return format!("npm install -g {tool}"),
    };
    format!("npm install -g {pkg}@latest")
}

/// 工具生命周期动作：install / update。
#[derive(Debug, Clone, Copy)]
enum ToolLifecycleAction {
    Install,
    Update,
}

impl std::str::FromStr for ToolLifecycleAction {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "install" => Ok(Self::Install),
            "update" => Ok(Self::Update),
            _ => Err(AppError::Message(format!("unknown action: {s}"))),
        }
    }
}

/// 执行工具安装/更新动作。
///
/// **Web 模式风险提示**：在服务器上执行 `npm install -g`，需要：
/// 1. 服务器已安装 Node.js / npm
/// 2. 当前用户对全局 npm 目录有写权限（或配了 npm prefix）
/// 3. 前端 UI 应明确告知"将在服务器上执行 npm install"
pub async fn run_tool_lifecycle_action(tools: Vec<String>, action: String) -> Result<(), AppError> {
    let action: ToolLifecycleAction = action.parse()?;
    let requested: Vec<&str> = VALID_TOOLS
        .iter()
        .copied()
        .filter(|t| tools.iter().any(|s| s.as_str() == *t))
        .collect();
    if requested.is_empty() {
        return Err(AppError::Message("No supported tools selected".to_string()));
    }

    for tool in requested {
        let pkg = match tool {
            "claude" => "@anthropic-ai/claude-code",
            "codex" => "@openai/codex",
            "gemini" => "@google/gemini-cli",
            "grok" => "@xai/grok-cli",
            "opencode" => "opencode-ai",
            "openclaw" => "@openclaw/cli",
            "hermes" => "@hermes-agent/cli",
            _ => continue,
        };
        let action_str = match action {
            ToolLifecycleAction::Install => "install",
            ToolLifecycleAction::Update => "update",
        };
        log::info!("[tool-lifecycle] npm {action_str} -g {pkg}");

        let output = tokio::task::spawn_blocking(move || {
            Command::new("npm").args([action_str, "-g", pkg]).output()
        })
        .await
        .map_err(|e| AppError::Message(format!("npm spawn 失败: {e}")))?;

        match output {
            Ok(o) if o.status.success() => {
                log::info!("[tool-lifecycle] {tool} {action:?} 成功");
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                return Err(AppError::Message(format!(
                    "npm {action_str} -g {pkg} 失败: exit {}: {}",
                    o.status,
                    stderr.trim()
                )));
            }
            Err(e) => {
                return Err(AppError::Message(format!(
                    "npm {action_str} -g {pkg} 失败: {e}"
                )));
            }
        }
    }
    Ok(())
}
