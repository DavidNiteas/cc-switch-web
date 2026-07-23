<div align="center">

# CC Switch Web

### CC Switch 的无头 Web 服务器 —— 在任何服务器上管理 AI 编码工具

[![Version](https://img.shields.io/badge/version-3.17.0-blue.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Docker%20%7C%20任何有%20Rust%20的%20OS-lightgrey.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Built with Axum](https://img.shields.io/badge/built%20with-Axum%200.7-blue.svg)](https://github.com/tokio-rs/axum)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**[CC Switch](README_original.md) 的无头版本** —— 原生运行在无头 Linux 服务器、云虚拟机、Docker 容器以及任何没有桌面 GUI 的环境中。

[English](README_WEB.md) | 中文 | [日本語](README_WEB_JA.md) | [原始 CC Switch 文档](README_original.md) | [更新日志](CHANGELOG.md)

</div>

---

## 为什么需要 CC Switch Web？

[CC Switch](README_original.md) 是一款桌面应用，通过 GUI 管理 AI 编码工具（Claude Code、Codex、Gemini CLI 等）。但如果你在**无头服务器**上运行——云虚拟机、Docker 容器、CI 运行器——没有桌面环境、没有 GTK、没有显示器怎么办？

**CC Switch Web** 就是答案。它提供：

- **零 GUI 依赖** —— 不需要 GTK、WebKitGTK、X11。纯 Rust + Axum HTTP 服务器，可在任何 Linux 服务器上运行。
- **相同的业务逻辑** —— 与桌面版共享完全相同的 `cc-switch-core` crate。供应商管理、代理、MCP、提示词、技能、用量统计——全部一致。
- **基于浏览器的 UI** —— 相同的 React 前端被嵌入单个可执行文件，由内置 HTTP 服务器提供。在任意浏览器中打开 `http://localhost:18180`。
- **100% 命令对等** —— 全部 265 个 Tauri 命令都有 Web 等价物：251 个真实实现、4 个前端 shim（文件对话框）、1 个重启端点、5 个永久兜底、2 个 no-op、2 个部分迁移（返回路径/命令字符串）。

<div align="center">

```
┌──────────────────────────────────────────────────────────────────┐
│                      浏览器（任意操作系统）                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │  React   │  │  TanStack│  │   Shim   │  │  <input>     │    │
│  │   (UI)   │──│  Query   │──│(Tauri API)│──│(文件对话框)   │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
└──────────────────────────┬───────────────────────────────────────┘
                           │ HTTP / SSE
┌──────────────────────────▼───────────────────────────────────────┐
│              cc-switch-web (Rust + Axum)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  /api/invoke │  │  /api/upload │  │  /api/restart        │  │
│  │  (265 命令)  │  │  /api/download│  │  （优雅关闭）        │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                           │                                      │
│  ┌────────────────────────▼──────────────────────────────────┐  │
│  │              cc-switch-core（与桌面版共享）               │  │
│  │  服务层 · 数据库 · 命令 · 代理 · 会话管理器                │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           │                                      │
│  ┌────────────────────────▼──────────────────────────────────┐  │
│  │              SQLite (~/.cc-switch/cc-switch.db)           │  │
│  └──────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

</div>

## 架构

CC Switch Web 采用**三层架构**：

| Crate | 用途 | 依赖 |
|-------|------|------|
| `cc-switch-core` | 纯 Rust 业务逻辑（服务、数据库、命令、代理） | 零 Tauri、零 GTK |
| `cc-switch-tauri` | 桌面壳（GUI、托盘、窗口、插件） | core + Tauri 2 |
| `cc-switch-web` | 无头 Web 服务器（HTTP API、静态文件） | core + Axum |

**核心设计**：`cc-switch-core` 在桌面版和 Web 版之间共享。所有业务逻辑都在这里。Web crate 是一层薄薄的 HTTP 包装，就像 Tauri crate 是一层薄薄的 GUI 包装。

## 快速开始

### 方式一：下载预编译二进制

```bash
# 下载并运行
chmod +x cc-switch-web
RUST_LOG=info ./cc-switch-web

# 在浏览器中打开
# http://127.0.0.1:18180
```

### 方式二：从源码编译

```bash
# 前置条件：Rust 1.85+、Node.js 18+、pnpm 10+

# 1. 构建前端
npx pnpm@10 install
npx pnpm@10 build:web    # 输出到 dist-web/

# 2. 构建后端（将 dist-web/ 嵌入可执行文件）
cargo build --release --bin cc-switch-web
# 单文件二进制：target/release/cc-switch-web

# 3. 运行
RUST_LOG=info ./target/release/cc-switch-web
```

### 方式三：Docker（计划中）

```dockerfile
FROM debian:bookworm-slim
COPY cc-switch-web /usr/local/bin/
CMD ["cc-switch-web"]
```

### 首次运行

```bash
# 启动服务器
RUST_LOG=info ./cc-switch-web

# 服务器在首次启动时创建 ~/.cc-switch/cc-switch.db
# 在浏览器中打开 http://127.0.0.1:18180

# 验证是否工作
curl http://127.0.0.1:18180/api/version
# {"version":"3.17.0"}

curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"get_providers","args":{"app":"claude"}}'
```

## API 参考

### 核心端点

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/invoke` | POST | 通用命令分发器（265 个命令） |
| `/api/version` | GET | 获取服务器版本 |
| `/api/info` | GET | 获取服务器信息（版本、配置目录、主目录） |
| `/api/upload` | POST | 上传文件（multipart），返回临时路径 |
| `/api/download/:filename` | GET | 下载服务器端文件 |
| `/api/restart` | POST | 触发优雅关闭 + systemd 重启 |
| `/api/events` | GET | SSE 事件流（计划中） |

### 命令调用

所有 CC Switch 命令通过 `POST /api/invoke` 调用：

```bash
# 获取设置
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"get_settings"}'

# 获取供应商
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"get_providers","args":{"app":"claude"}}'

# 启动代理服务器
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"start_proxy_server"}'
```

### 文件对话框（Web 模式）

文件对话框命令（`open_file_dialog`、`save_file_dialog`、`pick_directory`、`open_zip_file_dialog`）由**前端 shim** 使用 HTML `<input type="file">` 和 `<a download>` 处理。它们不经过 `/api/invoke`——而是 shim 将选中的文件上传到 `/api/upload`，返回服务器端临时路径。

## 部署

### systemd 服务

```ini
# /etc/systemd/system/cc-switch-web.service
[Unit]
Description=CC Switch Web 服务器
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/cc-switch-web
Environment=RUST_LOG=info
Restart=on-failure
RestartSec=2s
# /api/restart 端点触发优雅关闭；
# systemd 自动重启服务。

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now cc-switch-web
```

### 反向代理（Nginx）

```nginx
server {
    listen 80;
    server_name cc-switch.example.com;

    location / {
        proxy_pass http://127.0.0.1:18180;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # SSE 支持（实现后）
    location /api/events {
        proxy_pass http://127.0.0.1:18180;
        proxy_set_header Connection '';
        proxy_http_version 1.1;
        chunked_transfer_encoding off;
        proxy_buffering off;
    }
}
```

## 与桌面版的差异

| 功能 | 桌面版 (Tauri) | Web 版 |
|------|----------------|--------|
| 供应商管理 | ✅ | ✅ |
| 代理与故障转移 | ✅ | ✅ |
| MCP、提示词、技能 | ✅ | ✅ |
| 用量与成本追踪 | ✅ | ✅ |
| 会话管理器 | ✅ | ✅ |
| Copilot OAuth | ✅ | ✅ |
| 文件对话框 | 原生系统对话框 | 浏览器 `<input type="file">` |
| 系统托盘 | ✅ | ❌（无 GUI） |
| 应用自动更新 | Tauri updater | GitHub releases 检查 |
| 重启应用 | `app.restart()` | `/api/restart` + systemd |
| 轻量模式 | ✅ | ❌（桌面概念） |
| 打开文件夹/终端 | `opener` / `launch_terminal` | 返回路径/命令字符串（复制到剪贴板） |
| 系统浏览器 | `opener.open_url()` | `window.open()`（通过返回的 URL） |
| 主题 | 窗口主题 API | No-op（CSS 处理主题） |

**不变之处**：所有数据存储在相同的 `~/.cc-switch/cc-switch.db` SQLite 数据库中。相同的 `~/.claude/`、`~/.codex/`、`~/.gemini/` 配置文件以相同方式管理。

## 开发

### 前置条件

- Rust 1.85+
- Node.js 18+
- pnpm 10+

### 开发流程

```bash
# 1. 安装前端依赖
npx pnpm@10 install

# 2. 构建前端（Web 模式）
npx pnpm@10 build:web

# 3. 构建并运行后端
cargo build --bin cc-switch-web
RUST_LOG=info ./target/debug/cc-switch-web

# 4. 在浏览器中打开 http://127.0.0.1:18180
```

### 前端开发（热重载）

```bash
# 启动 Vite 开发服务器（端口 3000）
npx pnpm@10 dev:renderer -- --config vite.web.config.ts

# 另一个终端，启动 API 服务器
RUST_LOG=info ./target/debug/cc-switch-web
```

Vite 开发服务器会将 API 调用代理到 `:18180` 的后端。

### 测试

```bash
# Core 单元测试（1836 个测试）
cargo test -p cc-switch-core --all-targets

# Web crate 测试
cargo test -p cc-switch-web --all-targets

# 验证零 Tauri/GTK 依赖
cargo tree -p cc-switch-web -i tauri  # 应输出 "did not match"
cargo tree -p cc-switch-web -i gtk    # 应输出 "did not match"

# 前端构建
npx pnpm@10 build:web
```

### 项目结构

```
cc-switch-web/
├── Cargo.toml                    # Workspace 根目录
├── src-core/                     # 共享业务核心（无 Tauri/GTK）
│   └── src/
│       ├── commands/              # 265 个命令实现
│       ├── services/              # 业务逻辑（代理、供应商、MCP 等）
│       ├── database/              # SQLite DAO 层
│       ├── proxy/                 # 代理服务器、转发器、故障转移
│       ├── session_manager/       # 会话历史扫描
│       ├── codex_history_migration/
│       ├── deeplink/              # Deep link 导入
│       └── platform.rs            # Platform trait 抽象
├── src-tauri/                     # 桌面壳（Tauri 2）
│   └── src/                       # 薄包装，调用 core
├── src-web/                       # 无头 Web 服务器
│   └── src/
│       ├── main.rs                # 入口点（axum + 优雅关闭）
│       ├── routes.rs              # HTTP 路由 + 265 个命令分发器
│       └── platform_web.rs        # HeadlessPlatform 实现
├── src/                           # 前端（React + TypeScript）
│   ├── web/shims/                 # Tauri API Web 适配器
│   │   ├── core.ts                # invoke() → fetch /api/invoke
│   │   ├── event.ts               # listen() → EventSource
│   │   ├── plugin-dialog.ts        # HTML <input type="file">
│   │   ├── app.ts                 # getVersion() → /api/version
│   │   ├── window.ts              # no-op stub
│   │   └── path.ts                # homeDir/join → /api/info
│   └── ...                        # 与桌面版相同的 React 组件
├── vite.web.config.ts             # Web 前端构建配置
├── dist-web/                      # 构建时嵌入 cc-switch-web 的前端产物
└── _dev/                          # 架构文档与迁移路线图
```

### 关键设计：Platform Trait

`cc-switch-core` 中的 `Platform` trait 抽象了所有平台特定操作：

```rust
pub trait Platform: Send + Sync {
    async fn show_window(&self) -> Result<(), String>;
    async fn open_url(&self, url: &str) -> Result<(), String>;
    async fn copy_to_clipboard(&self, text: &str) -> Result<(), String>;
    fn app_version(&self) -> String;
    fn data_dir(&self) -> PathBuf;
    async fn restart_app(&self) -> Result<(), String>;
    fn emit_event(&self, event: &str, payload: serde_json::Value);
    // ...
}
```

- **桌面版**：`TauriPlatform` 通过 `AppHandle::emit()`、`tauri_plugin_dialog` 等实现。
- **Web 版**：`HeadlessPlatform` 通过 SSE 广播、`arboard` 剪贴板、no-op 窗口操作实现。

命令只依赖 `&dyn Platform`，不依赖 Tauri 类型。这就是同一个命令在两种模式下都能工作的原因。

## 常见问题

<details>
<summary><strong>可以在同一台机器上同时使用桌面版和 Web 版吗？</strong></summary>

可以。两个版本共享相同的 `~/.cc-switch/cc-switch.db` SQLite 数据库和相同的 `~/.claude/`、`~/.codex/`、`~/.gemini/` 配置文件。你可以在有 GUI 时使用桌面应用，SSH 登录时使用 Web 服务器。

</details>

<details>
<summary><strong>与桌面版相比缺了什么？</strong></summary>

Web 模式下只有 5 个命令永久不可用：
- `enter_lightweight_mode` / `exit_lightweight_mode` —— 桌面"迷你模式"概念
- `install_update_and_restart` —— 使用 systemd/docker 更新代替
- `launch_hermes_dashboard` —— 打开系统终端（无头服务器无终端）
- `set_window_theme` —— No-op（浏览器中由 CSS 处理主题）

其他一切功能完全相同。

</details>

<details>
<summary><strong>如何修改监听端口？</strong></summary>

目前端口硬编码在 `src-web/src/main.rs` 中为 `18180`。修改方式：

```rust
let listener = tokio::net::TcpListener::bind("127.0.0.1:18180")
```

未来版本将支持环境变量配置（`CC_SWITCH_WEB_PORT`）。

</details>

<details>
<summary><strong>安全吗？可以暴露到网络吗？</strong></summary>

默认情况下，服务器绑定到 `127.0.0.1`（仅本地访问）。要暴露到网络：

1. 在 `main.rs` 中修改绑定地址为 `0.0.0.0:18180`
2. 使用反向代理（Nginx/Caddy）配置 TLS + 认证
3. 考虑添加基于 Token 的认证（计划中的功能）

**不要**在未配置反向代理的情况下将原始服务器暴露到互联网。

</details>

<details>
<summary><strong>浏览器中的文件对话框怎么工作？</strong></summary>

前端 shim（`src/web/shims/plugin-dialog.ts`）拦截 `open_file_dialog` 和 `save_file_dialog` 命令。当你选择文件时，shim 会：

1. 创建一个 `<input type="file">` 元素
2. 用户在浏览器的原生文件选择器中选择文件
3. shim 将文件上传到 `POST /api/upload`
4. 服务器将其保存到 `/tmp/cc-switch-web-uploads/` 并返回路径
5. 路径被传递给后续命令（如 `import_config_from_file`）

这对前端业务代码是透明的——它只是调用 `invoke("open_file_dialog")` 并获得路径字符串，与 Tauri 一致。

</details>

## 贡献

欢迎提交 Issue 和建议！

提交 PR 前请确保：

```bash
# Core 测试通过
cargo test -p cc-switch-core --all-targets

# Web crate 编译通过
cargo check -p cc-switch-web

# 前端构建成功
npx pnpm@10 build:web

# Web crate 零 Tauri/GTK 依赖
cargo tree -p cc-switch-web -i tauri 2>&1 | grep "did not match"
cargo tree -p cc-switch-web -i gtk 2>&1 | grep "did not match"
```

## 许可证

MIT © Jason Young（原始 CC Switch）· Web 适配由 David Niteas 完成

## 致谢

- [CC Switch](README_original.md) by [Jason Young](https://github.com/farion1231) —— 原始桌面应用
- [Axum](https://github.com/tokio-rs/axum) —— Web 框架
- [Tauri](https://tauri.app/) —— 桌面框架（桌面版仍在使用）
