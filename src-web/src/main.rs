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

    // 创建 shutdown channel，供 /api/restart 端点触发优雅关闭
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
    routes::set_shutdown_sender(shutdown_tx);

    let app = routes::router(platform, app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:18180")
        .await
        .expect("failed to bind");

    log::info!("listening on http://127.0.0.1:18180");

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
