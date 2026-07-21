# cc-switch-web 改造架构计划（修订版）

> 更新时间：2026-07-20  
> 本版本替代 2026-07 初稿。初稿基于"MockRuntime + feature 分层"假设，经实验验证不可行，已转向"Core + Tauri + Web 三层分离"架构。

## 核心结论与转向原因

**底线要求**：cc-switch 的 Web/headless 版本必须能够在无头 Linux 设备上原生编译、原生运行，且不依赖 GTK/WebKitGTK 开发包。

**实验结果**：Tauri 2.11.x 在 Linux 目标下把 `gtk`、`muda`、`tauri-runtime` 中的 GTK/WebKitGTK 绑定写成了**非 optional 依赖**。即使设置 `tauri = { default-features = false, features = ["test"] }` 并关闭 `wry`，`cargo build` 仍会失败，报错需要 `gtk+-3.0`、`atk`、`pango`、`gdk-3.0`、`webkit2gtk-4.1` 等系统库。详细验证记录见 `_dev/verification-results.md`。

**转向**：原方案中"通过 Cargo feature 关闭 `wry` 即可去掉 GTK 依赖"的假设不成立。因此，Web/headless 路径必须**完全不依赖 `tauri` crate**，而不是试图在 Tauri 内部找无头模式。

新的方向是：

- 把业务核心下沉到独立的 `cc-switch-core` crate，零 Tauri 依赖；
- 桌面版 `cc-switch-tauri` 仅作为薄壳，依赖 core + Tauri；
- Web/headless 版 `cc-switch-web` 依赖 core + axum，可在无头 Linux 上干净编译运行。

## 最终目录结构（建议）

```
cc-switch/
├── src/                          # 前端（React + TypeScript）
│   ├── web/shims/               # 恢复/新建：Tauri API 的 Web shim
│   └── ...
├── src-core/                     # 新增：纯 Rust 业务核心（无 Tauri）
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # 导出 services、commands、Platform trait
│       ├── platform.rs          # Platform trait 定义
│       ├── commands_impl/       # 284 个命令的业务实现（普通 Rust 函数）
│       ├── services/            # 从 src-tauri 下移
│       ├── database/
│       ├── store.rs             # AppState
│       └── error.rs
├── src-tauri/                    # 改造：Tauri 桌面薄壳
│   ├── Cargo.toml               # 依赖 cc-switch-core + tauri
│   └── src/
│       ├── lib.rs               # run()、setup、GUI 初始化
│       ├── main.rs
│       ├── commands.rs          # #[tauri::command] 包装，调用 core 实现
│       └── platform_tauri.rs    # Tauri 版 Platform 实现
├── src-web/                      # 新增：独立无头 web 服务 crate
│   ├── Cargo.toml               # 依赖 cc-switch-core + axum（无 tauri）
│   ├── src/
│   │   ├── main.rs              # axum + HeadlessPlatform
│   │   ├── routes.rs            # HTTP/SSE 路由
│   │   └── platform_web.rs      # Headless 版 Platform 实现
│   └── build.rs / build.sh
├── vite.web.config.ts            # 恢复/新建：Web 前端构建配置
├── _dev/                         # 本文档与调研资料
└── package.json                  # 增加 build:web 脚本
```

## 三层职责

### 1. cc-switch-core（业务核心）

**设计原则**：
- 不依赖 `tauri`、`wry`、`gtk`、`webkit2gtk`；
- 不依赖任何桌面平台 GUI 库；
- 所有业务逻辑以普通 Rust 函数或 async 函数暴露；
- 平台上层能力通过 `Platform` trait 抽象。

**关键模块**：
- `store::AppState`：全局状态，仅包含 `db`、`proxy_service`、`usage_cache`；
- `services/`：现有服务层整体下移；
- `commands_impl/`：284 个命令的业务实现；
- `platform.rs`：`Platform` trait 定义。

**示例**：

```rust
// src-core/src/commands_impl/proxy.rs
pub async fn start_proxy_server(
    state: &AppState,
    _platform: &dyn Platform,
) -> Result<ProxyServerInfo, AppError> {
    state.proxy_service.start().await
}
```

### 2. cc-switch-tauri（桌面薄壳）

**职责**：
- 依赖 `cc-switch-core`；
- 实现 `TauriPlatform`（`Platform` trait 的 Tauri 版本）；
- 保留 `#[tauri::command]` 函数，但内部只调用 core 命令实现；
- 处理 GUI 初始化、托盘、窗口、deep-link、updater 等桌面专属逻辑。

**示例**：

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub async fn start_proxy_server(
    state: tauri::State<'_, AppState>,
    platform: tauri::State<'_, TauriPlatform>,
) -> Result<ProxyServerInfo, String> {
    cc_switch_core::commands_impl::proxy::start_proxy_server(&state, &*platform)
        .await
        .map_err(|e| e.to_string())
}
```

### 3. cc-switch-web（无头 Web 服务）

**职责**：
- 依赖 `cc-switch-core`，**不依赖 `tauri`**；
- 实现 `HeadlessPlatform`（`Platform` trait 的无头版本）；
- 用 axum 暴露 HTTP/SSE API；
- 静态服务 `dist-web/`。

**关键端点**：
- `POST /api/invoke`：通用命令调用；
- `GET /api/events`：SSE 事件推送；
- `GET /api/version`、`GET /api/info`：元信息；
- 其余路径静态服务前端产物。

## 平台抽象层（Platform trait）

这是连接桌面与无头的"专用中间层"。所有原生命令中直接调用的 Tauri API，都收敛到 `Platform` trait。

```rust
#[async_trait]
pub trait Platform: Send + Sync {
    // 窗口/托盘（桌面版实现，headless 版 no-op 或返回 Err）
    async fn show_window(&self) -> Result<(), String>;
    async fn hide_window(&self) -> Result<(), String>;
    async fn set_tray_tooltip(&self, text: &str) -> Result<(), String>;

    // 系统交互
    async fn open_url(&self, url: &str) -> Result<(), String>;
    async fn show_message(&self, title: &str, body: &str) -> Result<(), String>;
    async fn pick_file(&self) -> Result<Option<PathBuf>, String>;
    async fn copy_to_clipboard(&self, text: &str) -> Result<(), String>;

    // 应用生命周期
    fn app_version(&self) -> String;
    fn data_dir(&self) -> PathBuf;
    async fn restart_app(&self) -> Result<(), String>;
    async fn exit_app(&self, code: i32);

    // 事件
    fn emit_event(&self, event: &str, payload: serde_json::Value);
}
```

**设计要点**：
- `Platform` trait 只描述"能力"，不暴露 Tauri 类型；
- 桌面版通过 `TauriPlatform { app: AppHandle<tauri::Wry> }` 实现；
- headless 版通过 `HeadlessPlatform` 实现，窗口/托盘类方法返回 `Err("not available in headless mode")` 或 no-op；
- 命令实现只依赖 `&dyn Platform`，不感知后端形态。

## 命令迁移策略

### 分类处理

| 命令类型 | 数量估算 | 处理方式 |
|---------|---------|---------|
| 纯业务命令（只操作 AppState/services） | ~250 | 直接下移到 core，Tauri/web 均调用同一实现 |
| 平台上层命令（打开链接、剪贴板、对话框等） | ~23 | 业务逻辑下移到 core，内部调用 `Platform` trait |
| GUI 专用命令（窗口、托盘、主题等） | ~10 | core 中保留 no-op/Err 实现，桌面版实际生效 |

### 迁移顺序

建议先迁移与 GUI 无关的命令（proxy、settings、provider、config 等），再处理涉及 `Platform` 的命令。先验证端到端可行，再批量迁移剩余命令。

## 前端分层

### 已有 shim（保留并完善）

`src/web/shims/*.ts` 已覆盖前端用到的全部 Tauri API：

- `core.ts`：invoke → `/api/invoke`
- `event.ts`：listen → `EventSource('/api/events')`
- `app.ts`：getVersion → `/api/version`
- `window.ts`：getCurrentWindow → no-op stub
- `path.ts`：homeDir/join → `/api/info` + 字符串拼接
- `plugin-process.ts`：exit → `/api/exit`
- `plugin-dialog.ts`：message → alert
- `plugin-updater.ts`：check → `/api/check_update`

### 构建配置

`vite.web.config.ts`：
- `resolve.alias` 把 `@tauri-apps/*` 指向 `src/web/shims/*.ts`；
- 禁用 dev-only 插件；
- 输出到 `dist-web/`。

`package.json` 增加：

```json
"build:web": "vite build --config vite.web.config.ts"
```

## 编译流程

### 开发/调试

```bash
# 1. 前端
pnpm install
pnpm build:web

# 2. 无头后端（无 Tauri，无 GTK）
cd src-web
cargo run --bin cc-switch-web

# 3. 桌面后端（验证回归）
cd src-tauri
cargo build
```

### 生产构建

```bash
cd src-web
cargo build --release --bin cc-switch-web
# 二进制在 target/release/cc-switch-web
# 运行时读取 dist-web/ 或编译期嵌入（后续可用 rust-embed）
```

## 风险与待验证

1. **core 与 tauri 的边界划分**：`lib.rs` 中有大量 setup、state manage、插件初始化，需要仔细拆分哪些属于 core，哪些属于 Tauri。
2. **事件系统**：Tauri 的 `Emitter` 和 headless 的 SSE 事件模型不同，需要统一抽象。
3. **State 生命周期**：`AppState` 和各类 `*State` 在 core 中如何初始化、在 axum 中如何注入，需要设计清楚。
4. **异步命令并发**：axum handler 中调用 core async 函数是否需要 `spawn_blocking`，取决于 core 内部是否有阻塞调用。
5. **桌面版回归**：改造后必须保证 `cargo check --all-targets` 通过，桌面版功能无损。

## 下一步行动

1. **创建最小 POC**：选 5 个命令（如 `start_proxy_server`、`get_settings`、`open_external`、`copy_text_to_clipboard`、`get_version`），验证 core + Platform trait + axum 端到端可行。
2. **创建 `src-core` crate**：把 `services/`、`database/`、`store.rs`、`error.rs` 等无 Tauri 模块迁移进去。
3. **定义 `Platform` trait**：列出所有需要抽象的原生 Tauri 能力。
4. **改造 `src-tauri` 为薄壳**：依赖 core，命令函数只保留 `#[tauri::command]` 包装。
5. **创建 `src-web` crate**：实现 headless Platform + axum 路由。
6. **恢复前端 shim 和 `vite.web.config.ts`**。
7. **批量迁移剩余命令**：按"纯业务 → 平台能力 → GUI 专用"顺序进行。
8. **回归测试**：桌面版与 Web 版并行验证，直到 284 个命令全部可用。
