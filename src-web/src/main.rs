mod platform_web;
mod routes;
mod web_assets;

use crate::platform_web::HeadlessPlatform;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // 初始化日志系统
    // 优先级：RUST_LOG > CC_SWITCH_LOG_LEVEL > 默认 "info"（生产级别）
    let log_level = std::env::var("RUST_LOG")
        .or_else(|_| std::env::var("CC_SWITCH_LOG_LEVEL"))
        .unwrap_or_else(|_| "info".to_string());
    std::env::set_var("RUST_LOG", &log_level);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let version = env!("CARGO_PKG_VERSION").to_string();

    let banner = format!(
        "\n\
        ╔══════════════════════════════════════════╗\n\
        ║         CC Switch Web v{version}          ║\n\
        ║     Headless API Server                  ║\n\
        ╚══════════════════════════════════════════╝"
    );
    log::info!("{}", banner);
    log::info!("log level: {log_level}");

    // 清理上次遗留的上传临时文件
    cleanup_old_uploads();

    // 初始化 core：创建配置目录并打开 SQLite 数据库。
    let core_state = cc_switch_core::init(None, None).expect("failed to initialize cc-switch-core");
    let app_config_dir = core_state.app_config_dir;

    log::info!("app_config_dir: {}", app_config_dir.display());

    let platform: Arc<dyn cc_switch_core::platform::Platform> =
        Arc::new(HeadlessPlatform::new(app_config_dir.clone(), version));

    let app_state = cc_switch_core::AppState::new(core_state.db);
    app_state.proxy_service.set_platform(platform.clone());

    let proxy_auth_state = cc_switch_core::proxy::ProxyAuthState::new();

    // 初始化全局代理 HTTP 客户端
    {
        let db = &app_state.db;
        let proxy_url = db.get_global_proxy_url().ok().flatten();
        if let Err(e) = cc_switch_core::proxy::http_client::init(proxy_url.as_deref()) {
            log::error!("[GlobalProxy] Failed to initialize with saved config: {e}");
            if proxy_url.is_some() {
                log::warn!("[GlobalProxy] Clearing invalid proxy config from database");
                if let Err(clear_err) = db.set_global_proxy_url(None) {
                    log::error!("[GlobalProxy] Failed to clear invalid config: {clear_err}");
                }
            }
            if let Err(fallback_err) = cc_switch_core::proxy::http_client::init(None) {
                log::error!("[GlobalProxy] Failed to initialize direct connection: {fallback_err}");
            }
        }
    }

    // 创建 shutdown channel，供 /api/restart 端点触发优雅关闭
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
    routes::set_shutdown_sender(shutdown_tx);

    let app = routes::router(platform, app_state.clone(), proxy_auth_state);

    let port = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(18180);
    let addr = format!("127.0.0.1:{port}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    log::info!("────────────────────────────────────────");
    log::info!("  server started on http://{addr}");
    log::info!("  log level: {log_level} (set CC_SWITCH_LOG_LEVEL=debug for verbose)");
    log::info!("────────────────────────────────────────");

    // 后台启动初始化流程
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        routes::startup_initialization(app_state_clone).await;
    });

    let proxy_service = app_state.proxy_service.clone();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = shutdown_rx.recv() => log::info!("[shutdown] triggered by /api/restart"),
                _ = tokio::signal::ctrl_c() => log::info!("[shutdown] triggered by SIGINT"),
                _ = wait_for_sigterm() => log::info!("[shutdown] triggered by SIGTERM"),
            }
            // 停止代理服务器（如果正在运行）
            log::info!("[shutdown] stopping proxy server...");
            if let Err(e) = proxy_service.stop().await {
                log::error!("[shutdown] failed to stop proxy server: {e}");
            } else {
                log::info!("[shutdown] proxy server stopped");
            }
            log::info!("[shutdown] goodbye");
        })
        .await
        .expect("server failed");
}

/// 监听 SIGTERM 信号（仅 Unix）。
#[cfg(unix)]
async fn wait_for_sigterm() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    sigterm.recv().await;
}

/// Unix 之外的平台永远不返回，让 select! 跳过此分支。
#[cfg(not(unix))]
async fn wait_for_sigterm() {
    std::future::pending::<()>().await;
}

/// 启动时清理 /tmp/cc-switch-web-uploads/ 中超过 24 小时的遗留文件。
fn cleanup_old_uploads() {
    let upload_dir = std::env::temp_dir().join("cc-switch-web-uploads");
    if !upload_dir.exists() {
        return;
    }
    let now = std::time::SystemTime::now();
    let max_age = std::time::Duration::from_secs(24 * 60 * 60);
    let mut removed: usize = 0;
    let mut failed: usize = 0;

    if let Ok(entries) = std::fs::read_dir(&upload_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > max_age {
                                if std::fs::remove_file(entry.path()).is_ok() {
                                    removed += 1;
                                } else {
                                    failed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if removed > 0 || failed > 0 {
        log::info!(
            "[cleanup] removed {removed} old upload files from {} (failed: {failed})",
            upload_dir.display()
        );
    }
}
