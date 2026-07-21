use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 对话框类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageDialogKind {
    Info,
    Warning,
    Error,
}

/// 文件过滤器。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// 文件对话框选项（打开/保存共用）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub default_path: Option<PathBuf>,
    pub filters: Vec<FileFilter>,
}

/// 平台能力抽象层。
///
/// 桌面版通过 Tauri 实现，无头版通过 HeadlessPlatform 实现。
/// 命令实现只依赖此 trait，不感知后端形态。
#[async_trait::async_trait]
pub trait Platform: Send + Sync {
    /// 应用版本号
    fn app_version(&self) -> String;

    /// 应用配置目录
    fn app_config_dir(&self) -> PathBuf;

    /// 用户主目录
    fn get_home_dir(&self) -> PathBuf;

    /// 在系统浏览器中打开 URL
    async fn open_url(&self, url: &str) -> Result<(), String>;

    /// 写入系统剪贴板
    async fn copy_to_clipboard(&self, text: &str) -> Result<(), String>;

    // ------------------------------------------------------------------
    // 对话框
    // ------------------------------------------------------------------

    /// 显示消息对话框
    async fn show_message(
        &self,
        title: &str,
        message: &str,
        kind: MessageDialogKind,
    ) -> Result<(), String>;

    /// 显示确认对话框，返回用户是否点击"确定"。
    async fn show_confirm(&self, title: &str, message: &str) -> Result<bool, String>;

    /// 显示文件选择对话框，返回选中的文件路径（用户取消则返回 `None`）。
    async fn pick_file(&self, options: FileDialogOptions) -> Result<Option<PathBuf>, String>;

    /// 显示文件保存对话框，返回保存的文件路径（用户取消则返回 `None`）。
    async fn save_file(&self, options: FileDialogOptions) -> Result<Option<PathBuf>, String>;

    // ------------------------------------------------------------------
    // 窗口
    // ------------------------------------------------------------------

    /// 显示并聚焦主窗口
    async fn show_window(&self) -> Result<(), String>;

    /// 隐藏主窗口
    async fn hide_window(&self) -> Result<(), String>;

    /// 关闭主窗口
    async fn close_window(&self) -> Result<(), String>;

    /// 设置主窗口标题
    async fn set_window_title(&self, title: &str) -> Result<(), String>;

    // ------------------------------------------------------------------
    // 系统
    // ------------------------------------------------------------------

    /// 重启应用
    async fn restart_app(&self) -> Result<(), String>;

    /// 退出应用
    async fn exit_app(&self, code: i32) -> Result<(), String>;

    // ------------------------------------------------------------------
    // 事件
    // ------------------------------------------------------------------

    /// 向前端发送事件
    fn emit_event(&self, event: &str, payload: serde_json::Value);

    /// 监听前端发来的事件。
    ///
    /// 桌面版映射到 Tauri 的 `app.listen`；无头版由 SSE/HTTP 层注入。
    fn listen_event(&self, event: &str, handler: Box<dyn Fn(serde_json::Value) + Send + Sync>);
}
