mod platform_web;
mod routes;

use crate::platform_web::HeadlessPlatform;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // 初始化 core：创建配置目录并打开 SQLite 数据库。
    let core_state =
        cc_switch_core::init(None, None).expect("failed to initialize cc-switch-core");
    let app_config_dir = core_state.app_config_dir;
    let version = env!("CARGO_PKG_VERSION").to_string();

    log::info!("cc-switch-web starting");
    log::info!("version: {version}");
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:18180")
        .await
        .expect("failed to bind");

    log::info!("listening on http://127.0.0.1:18180");

    // 后台启动初始化流程
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        routes::startup_initialization(app_state_clone).await;
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            // 等待 /api/restart 或 ctrl_c 触发
            tokio::select! {
                _ = shutdown_rx.recv() => log::info!("[restart] graceful shutdown triggered by /api/restart"),
                _ = tokio::signal::ctrl_c() => log::info!("[restart] graceful shutdown triggered by SIGINT"),
            }
        })
        .await
        .expect("server failed");
}
