# mzparse 双模式分析

## 参考仓库

`/mnt/data/daiql/dev_repo/MetaEngine-mono/mzparse`

该仓库是用户此前自己开发的 Rust + Tauri 项目，已实现桌面端 + Web 服务端双模式，值得作为改造模板。

## 目录结构

```
mzparse/
├── apps/viewer-ui/            # 前端（Vue/React 类 SPA）
└── rust/crates/
    ├── mzparse-core/          # 纯 Rust 业务核心（UI 无关）
    └── mzparse-tauri/         # UI 壳层：Tauri 桌面 or axum Web
        ├── Cargo.toml         # desktop / web feature
        ├── src/
        │   ├── commands.rs    # 普通函数，被两种模式薄壳包裹
        │   ├── service.rs     # AppService，UI 无关业务逻辑
        │   ├── desktop.rs     # Tauri 桌面入口
        │   ├── web_server.rs  # axum Web 服务
        │   ├── lib.rs         # feature-gated 模块导出
        │   └── main.rs        # 根据 feature 选择入口
        └── build.rs
```

## 关键设计

### 1. UI 无关的 service 层

`service.rs` 提供 `AppService`，所有业务逻辑都在这里。命令函数不依赖 tauri 类型：

```rust
pub fn list_entries(service: Arc<AppService>) -> Result<Vec<EntrySummary>, String> {
    service.list_entries()
}
```

### 2. Feature 驱动的编译切换

`mzparse-tauri/Cargo.toml`：

```toml
[features]
desktop = ["dep:tauri", "tauri/wry", "dep:tauri-plugin-dialog", "dep:tauri-plugin-shell", ...]
web     = ["dep:axum", "dep:tokio", "dep:rust-embed", ...]

[dependencies]
tauri = { version = "2.0", optional = true }
tauri-plugin-dialog = { version = "2.0", optional = true }
axum  = { version = "0.7", optional = true }
```

**重点**：`tauri/wry` 是一个 feature。`web` feature 不启用 `wry`，因此 Web 构建完全不编译 WebKitGTK，这是它能做到"不依赖 GTK"的根本原因。

### 3. 命令双壳

- 桌面：`#[tauri::command]` 调用 `commands::list_entries(service)`。
- Web：axum route 调用同一个 `commands::list_entries(service)`。

### 4. 前端单一适配器

`apps/viewer-ui/src/api/tauri.ts`：

```ts
export const isTauri = () => !!(window as any).__TAURI__?.core;

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return isTauri() ? tauriInvoke<T>(cmd, args) : webInvoke<T>(cmd, args);
}
```

桌面走 `window.__TAURI__.core.invoke`，Web 走 `fetch('/api/invoke')`。事件同理：桌面走 `listen`，Web 走 `EventSource`。

所有组件只依赖这个适配器，不直接 import Tauri API。

### 5. 静态资源嵌入

Web 模式用 `rust-embed` 把前端 `dist/` 嵌进二进制，生成单文件可执行程序。

## 可借鉴到 cc-switch 的部分

1. **Feature 分层**：通过 `tauri/wry` feature 控制是否编译 WebKitGTK。这是解决"干净编译"问题的核心。
2. **前端统一适配器**：cc-switch 已用 vite alias + shim 实现等价效果，且不需要改 31 个调用文件，更省力。
3. **独立 crate / 独立入口**：把 headless/web 代码放到 `src-web/`，和 `src-tauri/` 解耦。
4. **rust-embed 打包**：后续可以让 `src-web` 的二进制自带前端资源，单文件分发。

## 不能直接照抄的地方

mzparse 的命令层是围绕普通函数 + `Arc<AppService>` 设计的，所以可以完全不用 tauri 也能跑 Web 模式。

cc-switch 的 284 个命令已经写成 `#[tauri::command]` + `tauri::State<'_, AppState>` 注入。如果强行改成 mzparse 模式，需要重写所有命令签名和调用链，工程量大且极易破坏桌面版。

因此 cc-switch 应采用**杂交方案**：

- **后端桥接仍用 MockRuntime + 真实 invoke_handler**（保留 284 命令原样）。
- **编译流程借鉴 mzparse 的 feature 分层**：给 `src-tauri` 增加 `desktop` feature，Web 构建关闭 `wry`，从而去掉 GTK 依赖。
- **前端复用现有 shim 层**，等效于 mzparse 的适配器。

这样既能得到 mzparse 的干净编译流程，又不必重写命令层。
