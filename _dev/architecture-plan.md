# cc-switch-web 改造架构计划

## 核心目标

在不改动 cc-switch 原有 GUI 和业务功能的前提下，实现后端与前端的彻底解耦：

- 前端既可以是 Tauri 桌面窗口，也可以是浏览器里的 Web GUI。
- 后端既可以是 Tauri 壳，也可以是独立 headless web 服务。
- 命令层代码复用率最大化。

## 最终目录结构（建议）

```
cc-switch/
├── src/                         # 前端（React）
│   ├── web/shims/              # 新增：Tauri API 的 Web shim
│   └── ...
├── src-tauri/                   # 原 Tauri 后端
│   ├── Cargo.toml              # 改为 feature 分层
│   └── src/
│       ├── lib.rs              # 增加 pub mod 导出 + feature gate GUI 代码
│       ├── commands/           # 保持原样，由 generate_handler! 注册
│       ├── ...                 # 其他模块保持原样
│       └── bin/                # 删除，迁到 src-web
├── src-web/                     # 新增：独立无头 web 服务 crate
│   ├── Cargo.toml              # 依赖 src-tauri（default-features = false）
│   ├── src/
│   │   └── main.rs             # axum + MockRuntime 桥
│   └── build.rs / build.sh     # 编译脚本（处理镜像、GTK stub 等）
├── vite.web.config.ts          # 新增：Web 前端构建配置
├── _dev/                        # 本文档与调研资料
└── package.json                # 增加 build:web 脚本
```

## 后端分层

### 1. src-tauri 改造

**Cargo.toml**

```toml
[features]
default = ["desktop"]
desktop = ["tauri/wry", "tauri/tray-icon", "tauri/image-png", "dep:tauri-plugin-dialog", ...]

[dependencies]
tauri = { version = "2.8.2", default-features = false, features = ["protocol-asset"] }
# 插件全部 optional，只在 desktop feature 启用
```

注意：具体保留哪些 tauri 基础 feature 需要实测。MockRuntime 至少需要 `test` feature（或默认开启）。桌面版需要 `wry`、`tray-icon`、`image-png` 等。

**src-tauri/src/lib.rs**

- 将 `mod commands;` 等改为 `pub mod commands;`，让外部 crate 能拿到命令函数路径。
- 给 `run()` 及 GUI 相关代码加 `#[cfg(feature = "desktop")]`：
  - tray 创建
  - 窗口创建
  - 23 个 Wry 专用命令（可改为 `AppHandle<R: Runtime>` 泛型，或保持 desktop-only）
  - `on_window_event`、deep-link 插件、updater 插件等
- 将非 GUI 初始化逻辑尽量抽成独立 `pub fn init_headless(app: &mut App<R>)` 函数，供 `src-web` 调用。
- 命令清单生成：建议用 `#[macro_export] macro_rules! all_commands { ... }` 共享给 desktop 和 web，避免两份清单。

### 2. src-web 新建

`src-web/Cargo.toml`：

```toml
[package]
name = "cc-switch-web"
version = "3.17.0"
edition = "2021"

[dependencies]
cc-switch = { path = "../src-tauri", default-features = false }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
axum = "0.7"
tower-http = { version = "0.5", features = ["cors"] }
serde_json = "1.0"
```

关键：`cc-switch` 关闭 `desktop` 特性 → 不编译 `wry` → 不需要 GTK/WebKitGTK 开发包。

**src-web/src/main.rs**：

1. 用 `tauri::test::mock_builder()` 构建 App。
2. 调用 `cc_switch_lib::init_headless(&mut app)` 做数据层初始化。
3. 用 `cc_switch_lib::all_commands!` 生成 `invoke_handler` 注册到 mock App。
4. `app.run_iteration()` 触发 setup。
5. 创建 mock webview。
6. 启动 axum：
   - `POST /api/invoke` → `get_ipc_response`
   - `GET /api/events` → SSE 事件桥
   - `GET /api/version`、`GET /api/info`
   - `POST /api/exit`
   - 其余路径静态服务 `dist-web/`

### 3. 命令覆盖率

默认情况下 261/284 命令可直接工作。剩余 23 个 Wry 命令可选处理：

- 方案 A（简单）：保持 desktop-only，Web 下返回 "Command not found"，前端有兜底不白屏。
- 方案 B（推荐）：把签名改成泛型 `AppHandle<R: Runtime>`，Web 下也能注册；无头运行时它们调用窗口/托盘 API 会失败，但能通过 Result 优雅返回错误。

## 前端分层

### 已有 shim（保留）

`src/web/shims/*.ts` 已覆盖前端用到的全部 Tauri API：

- `core.ts`：invoke → `/api/invoke`
- `event.ts`：listen → `EventSource('/api/events')`
- `app.ts`：getVersion → `/api/version`
- `window.ts`：getCurrentWindow → no-op stub
- `path.ts`：homeDir/join → `/api/info` + 字符串拼接
- `plugin-process.ts`：exit → `/api/exit`
- `plugin-dialog.ts`：message → alert
- `plugin-updater.ts`：后续可补 check → `/api/check_update`

### 构建配置

`vite.web.config.ts`：

- resolve.alias 把 `@tauri-apps/*` 指向 `src/web/shims/*.ts`
- 禁用 dev-only 插件（如 code-inspector-plugin）
- 输出到 `dist-web/`

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

# 2. 后端（无 GTK，无代理）
cd src-web
cargo run --bin cc-switch-web
```

### 生产构建

```bash
cd src-web
cargo build --release --bin cc-switch-web
# 二进制在 target/release/cc-switch-web
# 运行时读取 dist-web/ 或编译期嵌入
```

### 镜像源

如果拉依赖慢或被代理干扰，可在项目内新建 `.cargo/config.toml`：

```toml
[registries.crates-io]
protocol = "sparse"

[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
```

## 风险与待验证

1. **tauri 无 wry 能否编译 MockRuntime**：这是整个方案最大的未知数。需要在最小原型上验证 `tauri = { default-features = false, features = ["test"] }` 在 Linux 上能否 `cargo build` 通过。
2. **state 初始化完整性**：`init_headless` 必须和原 `run()` 的 setup 非 GUI 部分完全一致，否则命令会 panic。
3. **异步命令在 spawn_blocking 里的行为**：需确认 `get_ipc_response` 在并发请求下稳定。
4. **事件顺序**：后端 emit 事件时前端 SSE 必须已连接，否则可能漏早期事件。

## 下一步行动

1. 撤销当前实验性 headless 改动，让仓库恢复干净。
2. 创建最小验证 crate，测试 `tauri` 在 `default-features = false + test` 下无 GTK 编译是否可行。
3. 若验证通过，开始改造 `src-tauri/Cargo.toml` 和 `src-tauri/src/lib.rs` 的 feature 分层。
4. 创建 `src-web` crate，迁移 headless 代码。
5. 前端 shim 和 vite.web.config.ts 已基本就绪，只需整理进新结构。
