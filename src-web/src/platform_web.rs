use cc_switch_core::platform::{FileDialogOptions, MessageDialogKind, Platform};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

type EventListener = Box<dyn Fn(serde_json::Value) + Send + Sync>;
type EventListeners = Arc<RwLock<HashMap<String, Vec<EventListener>>>>;

pub struct HeadlessPlatform {
    app_config_dir: PathBuf,
    version: String,
    listeners: EventListeners,
}

impl HeadlessPlatform {
    pub fn new(app_config_dir: PathBuf, version: String) -> Self {
        Self {
            app_config_dir,
            version,
            listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Platform for HeadlessPlatform {
    fn app_version(&self) -> String {
        self.version.clone()
    }

    fn app_config_dir(&self) -> PathBuf {
        self.app_config_dir.clone()
    }

    fn get_home_dir(&self) -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    }

    async fn open_url(&self, _url: &str) -> Result<(), String> {
        Err("open_url is not available in headless mode".to_string())
    }

    async fn copy_to_clipboard(&self, text: &str) -> Result<(), String> {
        let text = text.to_string();
        tokio::task::spawn_blocking(move || {
            let mut clipboard =
                arboard::Clipboard::new().map_err(|e| format!("访问系统剪贴板失败: {e}"))?;
            clipboard
                .set_text(text)
                .map_err(|e| format!("写入系统剪贴板失败: {e}"))?;
            Ok::<_, String>(())
        })
        .await
        .map_err(|e| format!("剪贴板任务执行失败: {e}"))?
    }

    async fn show_message(
        &self,
        title: &str,
        message: &str,
        kind: MessageDialogKind,
    ) -> Result<(), String> {
        log::info!(
            "[headless dialog] show_message({:?}): {} - {}",
            kind,
            title,
            message
        );
        Ok(())
    }

    async fn show_confirm(&self, title: &str, message: &str) -> Result<bool, String> {
        log::info!("[headless dialog] show_confirm: {} - {}", title, message);
        // 无头环境没有用户交互，默认返回 false，避免误操作。
        Ok(false)
    }

    async fn pick_file(&self, _options: FileDialogOptions) -> Result<Option<PathBuf>, String> {
        Err("pick_file is not available in headless mode".to_string())
    }

    async fn save_file(&self, _options: FileDialogOptions) -> Result<Option<PathBuf>, String> {
        Err("save_file is not available in headless mode".to_string())
    }

    async fn show_window(&self) -> Result<(), String> {
        Err("show_window is not available in headless mode".to_string())
    }

    async fn hide_window(&self) -> Result<(), String> {
        Err("hide_window is not available in headless mode".to_string())
    }

    async fn close_window(&self) -> Result<(), String> {
        Err("close_window is not available in headless mode".to_string())
    }

    async fn set_window_title(&self, title: &str) -> Result<(), String> {
        log::info!("[headless window] set_window_title: {}", title);
        Ok(())
    }

    async fn restart_app(&self) -> Result<(), String> {
        Err("restart_app is not available in headless mode".to_string())
    }

    async fn exit_app(&self, code: i32) -> Result<(), String> {
        log::info!("[headless] exit_app called with code {code}");
        std::process::exit(code);
    }

    fn emit_event(&self, event: &str, payload: serde_json::Value) {
        log::info!("emit_event [{}]: {}", event, payload);
        if let Ok(listeners) = self.listeners.read() {
            if let Some(handlers) = listeners.get(event) {
                for handler in handlers {
                    handler(payload.clone());
                }
            }
        }
    }

    fn listen_event(&self, event: &str, handler: Box<dyn Fn(serde_json::Value) + Send + Sync>) {
        if let Ok(mut listeners) = self.listeners.write() {
            listeners
                .entry(event.to_string())
                .or_default()
                .push(handler);
        }
    }
}
