//! 使用统计实时刷新事件模块。
//!
//! 当 `proxy_request_logs` 表写入新数据时（代理日志、会话同步、归档等），
//! 通过本模块向前端推送 `usage-log-recorded` 事件，让 UsageDashboard
//! 立刻 invalidate 查询缓存而无需等待轮询周期。
//!
//! 设计要点：
//! - 回调注入：core 不依赖 Tauri 事件系统，由 tauri/web 外壳在初始化时
//!   注入实际的 emit 闭包（tauri 走 `AppHandle::emit`，web 走 SSE 广播）。
//! - 200ms 防抖合并：流式响应等场景在短时间内可能写入多条日志，
//!   合并成一次事件可避免前端连续 invalidate。
//! - 不阻塞写入：通知失败仅记录 warn 日志，不向上传播错误。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

/// 前端监听的事件名
pub const EVENT_USAGE_LOG_RECORDED: &str = "usage-log-recorded";

/// 防抖窗口：合并 200ms 内的多次通知。
const DEBOUNCE_WINDOW: Duration = Duration::from_millis(200);

type EmitCallback = Arc<dyn Fn() + Send + Sync + 'static>;

static EMIT_CALLBACK: OnceLock<EmitCallback> = OnceLock::new();

/// 防抖标记：true 表示已有调度任务在等待 emit，后续通知合并到该任务。
static EMIT_SCHEDULED: AtomicBool = AtomicBool::new(false);

/// 在应用初始化阶段调用一次，注入实际的事件发射回调。
///
/// 重复调用是无害的（OnceLock 仅首次写入生效），但应用启动期只该被
/// `lib.rs::run` 或 `main.rs` 调一次。
///
/// 桌面版注入 `|| handle.emit("usage-log-recorded", ())`；
/// 无头 Web 版可注入 SSE 广播闭包或 no-op。
pub fn init(callback: EmitCallback) {
    if EMIT_CALLBACK.set(callback).is_err() {
        log::debug!("usage_events::init 重复调用，已忽略");
    } else {
        log::info!("[usage-event] emit 回调已注入，事件推送启用");
    }
}

/// 通知前端有新的使用日志写入。
///
/// 调用方**不**需要持有任何 handle，可以从任意线程/任意写入路径调用。
/// 内部 200ms 防抖合并，绝不阻塞调用线程。
pub fn notify_log_recorded() {
    // 回调未注入（典型出现在单元测试或 setup 之前）：直接放弃。
    let Some(callback) = EMIT_CALLBACK.get() else {
        return;
    };

    // 已有调度任务：本次通知被合并到既有任务里，无需再起线程。
    if EMIT_SCHEDULED.swap(true, Ordering::AcqRel) {
        return;
    }

    let callback = Arc::clone(callback);
    std::thread::spawn(move || {
        std::thread::sleep(DEBOUNCE_WINDOW);
        // 必须先清标志再调用回调：万一回调期间又有新通知进来，
        // 下一轮防抖窗口会重新调度，不会丢失。
        EMIT_SCHEDULED.store(false, Ordering::Release);

        // 调用注入的回调；不传播错误，仅 warn。
        // 使用 catch_unwind 防止 panic 跨线程污染调用方。
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (callback)())) {
            Ok(()) => {}
            Err(_) => {
                log::warn!("usage_events emit callback panicked");
            }
        }
    });
}
