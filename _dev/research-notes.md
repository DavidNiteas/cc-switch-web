# cc-switch 无头化调研笔记

## 项目背景

- 目标仓库：`/home/daiql/cc-switch`
- 技术栈：Tauri 2 + React/TypeScript 前端 + Rust 后端
- 运行环境：无头 Linux（无显示器，不能启动 GTK/WebKitGTK）
- 目标：让前端既可以是桌面 GUI，也可以是 Web GUI，后端彻底解耦

## 原始代码结构

```
src/           # React 前端
src-tauri/     # Rust 后端（crate `cc-switch`，lib 名 `cc_switch_lib`）
  src/commands/*.rs   # 284 个 #[tauri::command]
  src/lib.rs          # run() 入口、generate_handler! 注册命令
  Cargo.toml          # tauri 默认特性（含 wry/GTK）
```

前端直接调用 `@tauri-apps/api/core` 的 `invoke`，共 31 处；其余用到的 Tauri API：
- `@tauri-apps/api/event`（listen）
- `@tauri-apps/api/app`（getVersion）
- `@tauri-apps/api/window`（getCurrentWindow）
- `@tauri-apps/api/path`（homeDir、join）
- `@tauri-apps/plugin-process`（exit）
- `@tauri-apps/plugin-dialog`（message）

## 调研过的方案

### 方案 A：Xvfb + noVNC

用虚拟显示器跑完整 Tauri 桌面应用，再用 VNC 投到浏览器。

- 优点：100% 功能保留，零代码改动。
- 缺点：本质是远程桌面，资源浪费大，不是真正的 Web 应用。
- 结论：仅作为备选，不作为主方案。

### 方案 B：重写 REST API 调用 services 层

把后端逻辑封装成 UI 无关的 service，前端改调 HTTP。

- 优点：干净、无头化最彻底。
- 缺点：284 个命令里 192 个依赖 `tauri::State<'_, AppState>` 注入；`State` 字段私有，无法在外部构造；基本等于重写半个后端。
- 结论：不可行（工程量太大且破坏原有业务代码）。

### 方案 C：`tauri::test::MockRuntime` + axum 桥接（已做出原型）

利用 Tauri 2.11 自带的 `tauri::test` 模块里的 MockRuntime。它不需要 GTK/显示器，却能 `build()` 出真实的 App 并创建 webview，再通过 `tauri::test::get_ipc_response` 把 HTTP 请求转成真实 IPC 调用。

关键代码形态：

```rust
let app = tauri::test::mock_builder()
    .invoke_handler(tauri::generate_handler![...])   // 复用 284 个命令 handler
    .setup(...)                                       // 镜像非 GUI 初始化
    .build(tauri::test::mock_context(tauri::test::noop_assets()))
    .unwrap();

let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
    .build()
    .unwrap();

let res = tauri::test::get_ipc_response(
    &webview,
    tauri::webview::InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        url: "http://tauri.localhost".parse().unwrap(),
        body: tauri::ipc::InvokeBody::Json(args),
        headers: Default::default(),
        invoke_key: tauri::test::INVOKE_KEY.to_string(),
    },
);
```

前端用 vite alias 把 `@tauri-apps/*` 模块替换成基于 `fetch`/`EventSource` 的 shim，映射到 `/api/invoke`、`/api/events` 等端点。

- 优点：
  - 284 个命令 handler 原样复用，业务逻辑零改动。
  - 261/284 命令可直接工作。
- 缺点：
  - 23 个命令签名为具体 Wry 类型（`AppHandle`/Window/TrayIcon），在 MockRuntime 下无法编译，必须排除。
  - `setup()` 里的 GUI 部分（托盘、窗口、对话框）无法执行，需要手动镜像非 GUI 初始化，否则 state 会缺失。
  - 编译期仍然链接 `wry` 特性，因此仍需要 GTK/WebKitGTK 的 pkg-config 检查（本机用 `.gtk-stub/` 伪造解决）。

## 已验证的事实

1. `tauri::test` 模块需要 tauri 开启 `test` feature（源码 `#[cfg(any(test, feature = "test"))]`），不是无条件编译。
2. MockRuntime 下必须手动 `app.run_iteration()` 一次才能触发 `setup` 回调，否则 state 不会被 manage。
3. `get_ipc_response` 是同步阻塞的（内部 `mpsc::sync_channel`），在 axum handler 中需要用 `tokio::task::spawn_blocking` 并加 Mutex 串行化 webview 访问。
4. 事件桥接：用 `app.listen_any` 或按名 `app.listen` 捕获后端 emit 的事件，再经 SSE 推给前端。
5. 桌面版命令泛化：若把 23 个 Wry 专用命令的签名改为泛型 `AppHandle<R: Runtime>`，它们也能注册到 MockRuntime 下。这是后续提高覆盖率的可行方向。

## 原型验证结果

- `cargo build --bin headless` 成功。
- `cargo check --all-targets`（含桌面 GUI 路径）通过，0 warning。
- curl 验证：`get_settings`、`get_providers`、`add_provider`、`switch_provider` 等返回真实数据。
- Playwright 验证主界面完整渲染，console 仅 1 条预期错误（窗口主题命令未注册）。
- 服务已跑在 `http://127.0.0.1:18180`。

## 关键教训

- 不要低估 Tauri 编译期对 GTK/WebKitGTK 的依赖。即使走 MockRuntime，只要 `Cargo.toml` 里 tauri 默认特性包含 `wry`，链接时仍需要这些开发包。
- 本机环境特殊：`rustup` 可用但系统 GTK dev 包不完整；`.gtk-stub/` 只是编译期 workaround。
- 代理环境会干扰 crates.io TLS 下载，需要 `unset *_proxy` 或设置国内镜像源。
