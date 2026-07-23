//! cc-switch-core
//!
//! 纯 Rust 业务核心，不依赖 Tauri/GTK，供桌面版与无头 Web 版共享。

use std::path::PathBuf;
use std::sync::Arc;

pub mod app_config;
pub mod app_store;
pub mod auto_launch;
pub mod claude_desktop_config;
pub mod claude_mcp;
pub mod claude_plugin;
pub mod codex_config;
pub mod codex_history_migration;
pub mod codex_state_db;
pub mod commands;
pub mod config;
pub mod database;
pub mod deeplink;
pub mod error;
pub mod gemini_config;
pub mod gemini_mcp;
pub mod grok_config;
pub mod hermes_config;
pub mod init_status;
pub mod mcp;
pub mod model_capabilities;
pub mod openclaw_config;
pub mod opencode_config;
pub mod platform;
pub mod prompt;
pub mod prompt_files;
pub mod provider;
pub mod provider_defaults;
pub mod proxy;
pub mod services;
pub mod session_manager;
pub mod settings;
pub mod store;
pub mod usage_events;
pub mod usage_script;

pub use app_config::AppType;
pub use app_store::{get_app_config_dir_override, set_app_config_dir_override};
pub use config::get_app_config_dir;
pub use database::Database;
pub use error::AppError;
pub use platform::{FileDialogOptions, FileFilter, MessageDialogKind, Platform};
pub use provider::{
    AuthBinding, AuthBindingSource, ClaudeDesktopMode, ClaudeDesktopModelRoute, ClaudeModelConfig,
    CodexChatReasoningConfig, CodexModelConfig, GeminiModelConfig, LocalProxyRequestOverrides,
    OpenCodeModel, OpenCodeModelLimit, OpenCodeProviderConfig, OpenCodeProviderOptions, Provider,
    ProviderManager, ProviderMeta, UniversalProvider, UniversalProviderApps,
    UniversalProviderModels, UsageData, UsageResult, UsageScript,
};
pub use provider_defaults::{infer_provider_icon, ProviderIcon, DEFAULT_PROVIDER_ICONS};
pub use store::AppState;

/// 数据库变更通知回调。
///
/// 桌面版注入 webdav/s3 自动同步通知；无头版可注入 SSE 刷新或无注入。
pub type DbChangeCallback = Box<dyn Fn(&str) + Send + Sync>;

/// Core 初始化后的运行时状态。
#[derive(Clone)]
pub struct CoreState {
    pub app_config_dir: PathBuf,
    pub db: Arc<Database>,
}

/// 初始化 cc-switch-core。
///
/// - 若传入了 `app_config_dir_override`，则将其设为覆盖路径。
/// - 创建应用配置目录（如果不存在）。
/// - 初始化 SQLite Database。
///
/// TODO(阶段七): 在此函数内初始化日志系统（无头模式）。
pub fn init(
    app_config_dir_override: Option<PathBuf>,
    db_change_callback: Option<DbChangeCallback>,
) -> Result<CoreState, AppError> {
    if let Some(dir) = app_config_dir_override {
        set_app_config_dir_override(Some(dir));
    }

    let app_config_dir = config::get_app_config_dir();
    std::fs::create_dir_all(&app_config_dir).map_err(|e| AppError::io(&app_config_dir, e))?;

    let db = Arc::new(Database::init_with_callback(db_change_callback)?);

    Ok(CoreState { app_config_dir, db })
}
