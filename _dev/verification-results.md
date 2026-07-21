# Tauri 无 GTK 编译验证结果

> 验证时间：2026-07-20  
> 验证人：Kimi Code CLI  
> 结论：Tauri 2.11.x 在 Linux 上无法通过 Cargo feature 关闭 GTK 依赖。

## 验证目的

验证核心假设：是否可以通过 `tauri = { default-features = false, features = ["test"] }` 在 Linux 上编译 Tauri 的 `MockRuntime`，从而避免 GTK/WebKitGTK 开发包依赖。

如果该假设成立，则 cc-switch 可以采用"MockRuntime + axum 桥接 + feature 分层"方案，在无头 Linux 上干净编译运行。

## 验证环境

- 操作系统：Linux
- Rust 版本：`rustc 1.90.0 (1159e78c4 2025-09-14)`
- Cargo 版本：`cargo 1.90.0 (840b83a10 2025-07-30)`
- Tauri 版本：`2.8.2`（实际解析为 `2.11.5`）
- GTK dev 包状态：
  - `gtk+-3.0`：未安装
  - `webkit2gtk-4.1`：未安装

## 验证步骤

### 1. 创建最小验证 crate

路径：`_dev/verify-tauri-compile/`

`Cargo.toml`：

```toml
[package]
name = "verify-tauri-compile"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2.8.2", default-features = false, features = ["test"] }
```

`src/main.rs`：

```rust
fn main() {
    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()));
    match app {
        Ok(_) => println!("App built successfully"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

### 2. 执行编译

```bash
unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY all_proxy
cargo build
```

### 3. 编译结果

**失败**。错误信息摘要：

```
The system library `atk` required by crate `atk-sys` was not found.
The system library `pango` required by crate `pango-sys` was not found.
The system library `gdk-3.0` required by crate `gdk-sys` was not found.
```

### 4. 依赖树分析

使用 `cargo tree -i <crate>` 追踪 GTK 依赖来源：

```
gtk v0.18.2
├── muda v0.19.3
│   └── tauri v2.11.5
├── tauri v2.11.5
├── tauri-runtime v2.11.3
│   └── tauri v2.11.5
└── webkit2gtk v2.0.2
    └── tauri-runtime v2.11.3
```

```
webkit2gtk v2.0.2
└── tauri-runtime v2.11.3
    └── tauri v2.11.5
```

```
muda v0.19.3
└── tauri v2.11.5
```

### 5. 源码确认

查阅 `~/.cargo/registry/src/index.crates.io-*/tauri-2.11.5/Cargo.toml`：

```toml
[target.'cfg(any(target_os = "linux", ...))'.dependencies.gtk]
version = "0.18"
features = ["v3_24"]
# 注意：没有 optional = true

[target.'cfg(any(target_os = "linux", ...))'.dependencies.muda]
version = "0.19"
features = ["serde", "gtk"]
default-features = false
# 注意：没有 optional = true，且显式启用 gtk feature

[target.'cfg(any(target_os = "linux", ...))'.dependencies.webkit2gtk]
version = "2"
features = ["v2_40"]
optional = true
# 只有 webkit2gtk 是 optional，被 wry feature 控制
```

查阅 `~/.cargo/registry/src/index.crates.io-*/tauri-runtime-2.11.3/Cargo.toml`：

```toml
[target.'cfg(any(target_os = "linux", ...))'.dependencies.gtk]
version = "0.18"
features = ["v3_24"]
# 非 optional

[target.'cfg(any(target_os = "linux", ...))'.dependencies.webkit2gtk]
version = "=2.0"
features = ["v2_40"]
# 非 optional
```

## 关键发现

1. **`gtk` 是 Tauri 2.11.x 在 Linux 上的强制依赖**，无法通过任何 feature 关闭。
2. **`tauri-runtime` 同样在 Linux 上强制依赖 `gtk` 和 `webkit2gtk`**，且均非 optional。
3. **`muda` 被 Tauri 强制引入，且启用 `gtk` feature**，进一步加深 GTK 依赖。
4. `webkit2gtk` 虽然 optional，但被 `tauri-runtime` 的非 optional `webkit2gtk` 抵消。
5. 即使设置 `default-features = false` 并只开启 `test` feature，上述依赖仍然全部生效。

## 结论

**Tauri 2.11.x 在 Linux 上无法剥离 GTK 依赖。** 任何保留 `tauri` crate 的方案（包括 MockRuntime、feature 分层、关闭 `wry`）都不能在干净的无头 Linux 环境上编译。

因此，cc-switch 的无头 Web 版本必须采用**完全不依赖 `tauri` crate** 的架构，即：

- 业务核心下沉到独立的 `cc-switch-core` crate；
- Web/headless 版本 `cc-switch-web` 仅依赖 `cc-switch-core` + `axum`；
- 桌面版本 `cc-switch-tauri` 依赖 `cc-switch-core` + `tauri`，保持原有桌面体验。

## 参考

- 本次验证的临时 crate 位于 `_dev/verify-tauri-compile/`，可重复执行验证。
- 新的架构方向详见 `_dev/architecture-plan.md`。
