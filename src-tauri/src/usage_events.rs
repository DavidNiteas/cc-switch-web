//! 使用统计实时刷新事件模块 — 注入 Tauri emit 回调到 core。
//!
//! See `cc_switch_core::usage_events` for the actual implementation.

use std::sync::Arc;

use tauri::{AppHandle, Emitter};

/// 在应用 setup 阶段调用一次，把 Tauri emit 闭包注入到 core。
pub fn init(handle: AppHandle) {
    let callback: Arc<dyn Fn() + Send + Sync + 'static> = Arc::new(move || {
        if let Err(e) = handle.emit(cc_switch_core::usage_events::EVENT_USAGE_LOG_RECORDED, ()) {
            log::warn!(
                "emit {} 失败: {e}",
                cc_switch_core::usage_events::EVENT_USAGE_LOG_RECORDED
            );
        }
    });
    cc_switch_core::usage_events::init(callback);
}

/// 通知前端有新的使用日志写入。
pub fn notify_log_recorded() {
    cc_switch_core::usage_events::notify_log_recorded();
}
