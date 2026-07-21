# mzparse 双模式分析（修订版）

> 更新时间：2026-07-20  
> 结合 Tauri 无 GTK 编译验证结果，重新评估 mzparse 模式对 cc-switch 的适用性。

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

1. **业务核心下沉到独立 crate**：`mzparse-core` 是 `cc-switch-core` 的直接参考。把 services、database、state 全部下移到无 Tauri crate。
2. **命令函数普通化**：mzparse 的命令是普通 Rust 函数，cc-switch 也应把 284 个命令的业务实现写成普通函数，再由 Tauri/axum 薄壳包装。
3. **前端统一适配器**：cc-switch 已用 vite alias + shim 实现等价效果，且不需要改 31 个调用文件，比 mzparse 的前端适配器更省力，应该保留。
4. **独立 crate / 独立入口**：把 headless/web 代码放到 `src-web/`，和 `src-tauri/` 解耦，符合 mzparse 的分 crate 思路。
5. **rust-embed 打包**：后续可以让 `src-web` 的二进制自带前端资源，单文件分发。

## 需要调整的部分

mzparse 能靠 feature 关闭 `wry` 来去掉 GTK，是因为它的 UI 壳层 crate 可以选择完全不依赖 `tauri`。但 cc-switch 如果保留桌面版，就必须在某个 crate 里依赖 Tauri。

**因此 cc-switch 不能简单复制 mzparse 的 feature 切换，而应采用分 crate 架构**：

- `cc-switch-core`：零 Tauri 依赖（对应 mzparse-core）；
- `cc-switch-tauri`：依赖 core + Tauri（仅桌面版编译）；
- `cc-switch-web`：依赖 core + axum（无头版编译）。

这样 `cc-switch-web` 永远不接触 Tauri，自然也就没有 GTK 依赖。

## 不能直接照抄的地方（更新）

mzparse 的命令层是围绕普通函数 + `Arc<AppService>` 设计的，所以可以完全不用 tauri 也能跑 Web 模式。

cc-switch 的 284 个命令已经写成 `#[tauri::command]` + `tauri::State<'_, AppState>` 注入。**如果强行改成 mzparse 模式，需要重写所有命令签名和调用链，工程量大且极易破坏桌面版**。

**修正后的做法**：

- 保留命令函数的 Tauri 签名（桌面版不破坏）；
- 但把每个命令的**业务实现**抽成 core 中的普通函数；
- 用 `Platform` trait 封装原生命令中直接调用的 Tauri API；
- Tauri 命令函数内部只保留薄壳调用。

这样既能得到 mzparse 的干净分层和解耦，又不必重写命令的调用链。迁移成本从"重写"降到"包装"。

## 结论

mzparse 的"core + UI 壳"思想是 cc-switch 无头化的正确模板。不同点在于：

- mzparse 用 feature 在同 crate 内切换 Tauri/axum；
- cc-switch 由于 Tauri 2.x 在 Linux 上无法剥离 GTK，必须用**分 crate 架构**让 Web 路径完全不依赖 Tauri。

cc-switch 的最终架构应命名为 **"Core + Tauri + Web 三层分离"**，并引入 `Platform` trait 作为原生命令能力的抽象层。
