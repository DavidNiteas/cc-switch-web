use std::path::PathBuf;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind as TauriMessageDialogKind};

pub struct TauriPlatform {
    app: tauri::AppHandle,
}

impl TauriPlatform {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl cc_switch_core::platform::Platform for TauriPlatform {
    fn app_version(&self) -> String {
        self.app.package_info().version.to_string()
    }

    fn app_config_dir(&self) -> PathBuf {
        crate::config::get_app_config_dir()
    }

    fn get_home_dir(&self) -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    }

    async fn open_url(&self, url: &str) -> Result<(), String> {
        self.app
            .opener()
            .open_url(url, None::<String>)
            .map_err(|e| format!("打开链接失败: {e}"))
    }

    async fn copy_to_clipboard(&self, text: &str) -> Result<(), String> {
        tokio::task::spawn_blocking({
            let text = text.to_string();
            move || {
                let mut clipboard =
                    arboard::Clipboard::new().map_err(|e| format!("访问系统剪贴板失败: {e}"))?;
                clipboard
                    .set_text(text)
                    .map_err(|e| format!("写入系统剪贴板失败: {e}"))?;
                Ok::<_, String>(())
            }
        })
        .await
        .map_err(|e| format!("剪贴板任务执行失败: {e}"))?
    }

    async fn show_message(
        &self,
        title: &str,
        message: &str,
        kind: cc_switch_core::platform::MessageDialogKind,
    ) -> Result<(), String> {
        let kind = match kind {
            cc_switch_core::platform::MessageDialogKind::Info => TauriMessageDialogKind::Info,
            cc_switch_core::platform::MessageDialogKind::Warning => TauriMessageDialogKind::Warning,
            cc_switch_core::platform::MessageDialogKind::Error => TauriMessageDialogKind::Error,
        };
        tokio::task::spawn_blocking({
            let app = self.app.clone();
            let title = title.to_string();
            let message = message.to_string();
            move || {
                let _ = app
                    .dialog()
                    .message(message)
                    .title(title)
                    .kind(kind)
                    .buttons(MessageDialogButtons::Ok)
                    .blocking_show();
                Ok::<_, String>(())
            }
        })
        .await
        .map_err(|e| format!("对话框任务执行失败: {e}"))?
    }

    async fn show_confirm(&self, title: &str, message: &str) -> Result<bool, String> {
        tokio::task::spawn_blocking({
            let app = self.app.clone();
            let title = title.to_string();
            let message = message.to_string();
            move || {
                Ok(app
                    .dialog()
                    .message(message)
                    .title(title)
                    .kind(TauriMessageDialogKind::Warning)
                    .buttons(MessageDialogButtons::OkCancel)
                    .blocking_show())
            }
        })
        .await
        .map_err(|e| format!("确认对话框任务执行失败: {e}"))?
    }

    async fn pick_file(
        &self,
        options: cc_switch_core::platform::FileDialogOptions,
    ) -> Result<Option<PathBuf>, String> {
        tokio::task::spawn_blocking({
            let app = self.app.clone();
            move || {
                let mut builder = app.dialog().file();
                if let Some(title) = options.title {
                    builder = builder.set_title(title);
                }
                if let Some(default_path) = options.default_path {
                    if default_path.is_dir() {
                        builder = builder.set_directory(default_path);
                    } else if let Some(parent) = default_path.parent() {
                        builder = builder.set_directory(parent);
                    }
                }
                for filter in options.filters {
                    builder = builder.add_filter(&filter.name, &filter.extensions);
                }
                Ok(builder.blocking_pick_file())
            }
        })
        .await
        .map_err(|e| format!("文件选择对话框任务执行失败: {e}"))?
    }

    async fn save_file(
        &self,
        options: cc_switch_core::platform::FileDialogOptions,
    ) -> Result<Option<PathBuf>, String> {
        tokio::task::spawn_blocking({
            let app = self.app.clone();
            move || {
                let mut builder = app.dialog().file();
                if let Some(title) = options.title {
                    builder = builder.set_title(title);
                }
                if let Some(default_path) = options.default_path {
                    if default_path.is_file() {
                        if let Some(name) = default_path.file_name() {
                            builder = builder.set_file_name(name.to_string_lossy().as_ref());
                        }
                    } else if default_path.is_dir() {
                        builder = builder.set_directory(default_path);
                    }
                }
                for filter in options.filters {
                    builder = builder.add_filter(&filter.name, &filter.extensions);
                }
                Ok(builder.blocking_save_file())
            }
        })
        .await
        .map_err(|e| format!("文件保存对话框任务执行失败: {e}"))?
    }

    async fn show_window(&self) -> Result<(), String> {
        tokio::task::spawn_blocking({
            let app = self.app.clone();
            move || {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                    #[cfg(target_os = "linux")]
                    {
                        crate::linux_fix::nudge_main_window(window.clone());
                    }
                }
                Ok::<_, String>(())
            }
        })
        .await
        .map_err(|e| format!("显示窗口任务执行失败: {e}"))?
    }

    async fn hide_window(&self) -> Result<(), String> {
        tokio::task::spawn_blocking({
            let app = self.app.clone();
            move || {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
                Ok::<_, String>(())
            }
        })
        .await
        .map_err(|e| format!("隐藏窗口任务执行失败: {e}"))?
    }

    async fn close_window(&self) -> Result<(), String> {
        tokio::task::spawn_blocking({
            let app = self.app.clone();
            move || {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.close();
                }
                Ok::<_, String>(())
            }
        })
        .await
        .map_err(|e| format!("关闭窗口任务执行失败: {e}"))?
    }

    async fn set_window_title(&self, title: &str) -> Result<(), String> {
        let title = title.to_string();
        tokio::task::spawn_blocking({
            let app = self.app.clone();
            move || {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_title(&title);
                }
                Ok::<_, String>(())
            }
        })
        .await
        .map_err(|e| format!("设置窗口标题任务执行失败: {e}"))?
    }

    async fn restart_app(&self) -> Result<(), String> {
        tauri::process::restart(&self.app.env());
        // restart 通常不会返回；为兼容不同签名保留兜底返回值。
        Ok(())
    }

    async fn exit_app(&self, code: i32) -> Result<(), String> {
        self.app.exit(code);
        Ok(())
    }

    fn emit_event(&self, event: &str, payload: serde_json::Value) {
        let _ = self.app.emit(event, payload);
    }

    fn listen_event(&self, event: &str, handler: Box<dyn Fn(serde_json::Value) + Send + Sync>) {
        let event = event.to_string();
        self.app.listen(event, move |e| {
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(e.payload()) {
                handler(payload);
            }
        });
    }
}
