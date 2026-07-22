<div align="center">

# CC Switch Web

### Headless Web Server for CC Switch — Manage AI Coding Tools on Any Server

[![Version](https://img.shields.io/badge/version-3.17.0-blue.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Docker%20%7C%20any%20OS%20with%20Rust-lightgrey.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Built with Axum](https://img.shields.io/badge/built%20with-Axum%200.7-blue.svg)](https://github.com/tokio-rs/axum)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**The headless companion of [CC Switch](README_original.md)** — runs natively on headless Linux servers, cloud VMs, Docker containers, and any environment without a desktop GUI.

English | [中文](README_WEB_ZH.md) | [日本語](README_WEB_JA.md) | [Original CC Switch README](README_original.md) | [Changelog](CHANGELOG.md)

</div>

---

## Why CC Switch Web?

[CC Switch](README_original.md) is a desktop app for managing AI coding tools (Claude Code, Codex, Gemini CLI, etc.) with a GUI. But what if you're running on a **headless server** — a cloud VM, a Docker container, a CI runner — where there's no desktop environment, no GTK, no display?

**CC Switch Web** is the answer. It provides:

- **Zero GUI Dependencies** — No GTK, no WebKitGTK, no X11. Pure Rust + Axum HTTP server. Runs on any Linux server.
- **Same Business Logic** — Shares the exact same `cc-switch-core` crate as the desktop version. Provider management, proxy, MCP, prompts, skills, usage stats — all identical.
- **Browser-Based UI** — The same React frontend, served by the built-in HTTP server. Open `http://localhost:18180` in any browser.
- **100% Command Parity** — All 265 Tauri commands have Web equivalents: 251 real implementations, 4 frontend shim (file dialogs), 1 restart endpoint, 5 permanent fallbacks, 2 no-op, 2 partial (return path/command strings).

<div align="center">

```
┌──────────────────────────────────────────────────────────────────┐
│                      Browser (any OS)                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │  React   │  │  TanStack│  │  Shims   │  │  <input>     │    │
│  │   (UI)   │──│  Query   │──│(Tauri API)│──│(file dialog) │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
└──────────────────────────┬───────────────────────────────────────┘
                           │ HTTP / SSE
┌──────────────────────────▼───────────────────────────────────────┐
│              cc-switch-web (Rust + Axum)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  /api/invoke │  │  /api/upload │  │  /api/restart        │  │
│  │  (265 cmds)  │  │  /api/download│  │  (graceful shutdown)│  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                           │                                      │
│  ┌────────────────────────▼──────────────────────────────────┐  │
│  │              cc-switch-core (shared with desktop)        │  │
│  │  Services · Database · Commands · Proxy · Session Manager │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           │                                      │
│  ┌────────────────────────▼──────────────────────────────────┐  │
│  │              SQLite (~/.cc-switch/cc-switch.db)          │  │
│  └──────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

</div>

## Architecture

CC Switch Web uses a **three-layer architecture**:

| Crate | Purpose | Dependencies |
|-------|---------|-------------|
| `cc-switch-core` | Pure Rust business logic (services, database, commands, proxy) | Zero Tauri, zero GTK |
| `cc-switch-tauri` | Desktop shell (GUI, tray, window, plugins) | core + Tauri 2 |
| `cc-switch-web` | Headless web server (HTTP API, static files) | core + Axum |

**Key design**: `cc-switch-core` is shared between desktop and web. All business logic lives there. The web crate is a thin HTTP wrapper, just like the Tauri crate is a thin GUI wrapper.

## Quick Start

### Option 1: Download Pre-built Binary

```bash
# Download and run
chmod +x cc-switch-web
RUST_LOG=info ./cc-switch-web

# Open in browser
# http://127.0.0.1:18180
```

### Option 2: Build from Source

```bash
# Prerequisites: Rust 1.85+, Node.js 18+, pnpm 10+

# 1. Build frontend
npx pnpm@10 install
npx pnpm@10 build:web    # outputs to dist-web/

# 2. Build backend
cargo build --release --bin cc-switch-web
# binary: target/release/cc-switch-web

# 3. Run
RUST_LOG=info ./target/release/cc-switch-web
```

### Option 3: Docker (planned)

```dockerfile
FROM debian:bookworm-slim
COPY cc-switch-web /usr/local/bin/
COPY dist-web/ /app/dist-web/
WORKDIR /app
CMD ["cc-switch-web"]
```

### First Run

```bash
# Start the server
RUST_LOG=info ./cc-switch-web

# The server creates ~/.cc-switch/cc-switch.db on first launch
# Open http://127.0.0.1:18180 in your browser

# Verify it works
curl http://127.0.0.1:18180/api/version
# {"version":"3.17.0"}

curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"get_providers","args":{"app":"claude"}}'
```

## API Reference

### Core Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/invoke` | POST | Universal command dispatcher (265 commands) |
| `/api/version` | GET | Get server version |
| `/api/info` | GET | Get server info (version, config dir, home dir) |
| `/api/upload` | POST | Upload file (multipart), returns temp path |
| `/api/download/:filename` | GET | Download server-side file |
| `/api/restart` | POST | Trigger graceful shutdown + systemd restart |
| `/api/events` | GET | SSE event stream (planned) |

### Command Invocation

All CC Switch commands are available via `POST /api/invoke`:

```bash
# Get settings
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"get_settings"}'

# Get providers
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"get_providers","args":{"app":"claude"}}'

# Start proxy server
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"start_proxy_server"}'
```

### File Dialog (Web Mode)

File dialog commands (`open_file_dialog`, `save_file_dialog`, `pick_directory`, `open_zip_file_dialog`) are handled by the **frontend shim** using HTML `<input type="file">` and `<a download>`. They don't go through `/api/invoke` — instead, the shim uploads the selected file to `/api/upload` and returns a server-side temp path.

## Deployment

### systemd Service

```ini
# /etc/systemd/system/cc-switch-web.service
[Unit]
Description=CC Switch Web Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/cc-switch-web
Environment=RUST_LOG=info
Restart=on-failure
RestartSec=2s
# The /api/restart endpoint triggers graceful shutdown;
# systemd auto-restarts the service.

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now cc-switch-web
```

### Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name cc-switch.example.com;

    location / {
        proxy_pass http://127.0.0.1:18180;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # SSE support (when implemented)
    location /api/events {
        proxy_pass http://127.0.0.1:18180;
        proxy_set_header Connection '';
        proxy_http_version 1.1;
        chunked_transfer_encoding off;
        proxy_buffering off;
    }
}
```

## Differences from Desktop Version

| Feature | Desktop (Tauri) | Web |
|---------|----------------|-----|
| Provider management | ✅ | ✅ |
| Proxy & failover | ✅ | ✅ |
| MCP, Prompts, Skills | ✅ | ✅ |
| Usage & cost tracking | ✅ | ✅ |
| Session manager | ✅ | ✅ |
| Copilot OAuth | ✅ | ✅ |
| File dialogs | Native OS dialog | Browser `<input type="file">` |
| System tray | ✅ | ❌ (no GUI) |
| App auto-updater | Tauri updater | GitHub releases check |
| Restart app | `app.restart()` | `/api/restart` + systemd |
| Lightweight mode | ✅ | ❌ (desktop concept) |
| Open folder/terminal | `opener` / `launch_terminal` | Returns path/command string (copy to clipboard) |
| System browser | `opener.open_url()` | `window.open()` (via returned URL) |
| Theme | Window theme API | No-op (CSS handles theme) |

**Unchanged**: All data is stored in the same `~/.cc-switch/cc-switch.db` SQLite database. The same `~/.claude/`, `~/.codex/`, `~/.gemini/` config files are managed identically.

## Development

### Prerequisites

- Rust 1.85+
- Node.js 18+
- pnpm 10+

### Development Workflow

```bash
# 1. Install frontend dependencies
npx pnpm@10 install

# 2. Build frontend (web mode)
npx pnpm@10 build:web

# 3. Build and run backend
cargo build --bin cc-switch-web
RUST_LOG=info ./target/debug/cc-switch-web

# 4. Open http://127.0.0.1:18180 in browser
```

### Frontend Development (Hot Reload)

```bash
# Start Vite dev server (port 3000)
npx pnpm@10 dev:renderer -- --config vite.web.config.ts

# In another terminal, start the API server
RUST_LOG=info ./target/debug/cc-switch-web
```

The Vite dev server proxies API calls to the backend at `:18180`.

### Testing

```bash
# Core unit tests (1836 tests)
cargo test -p cc-switch-core --all-targets

# Web crate tests
cargo test -p cc-switch-web --all-targets

# Verify zero Tauri/GTK dependencies
cargo tree -p cc-switch-web -i tauri  # should say "did not match"
cargo tree -p cc-switch-web -i gtk    # should say "did not match"

# Frontend build
npx pnpm@10 build:web
```

### Project Structure

```
cc-switch-web/
├── Cargo.toml                    # Workspace root
├── src-core/                     # Shared business core (no Tauri/GTK)
│   └── src/
│       ├── commands/              # 265 command implementations
│       ├── services/              # Business logic (proxy, provider, mcp, etc.)
│       ├── database/              # SQLite DAO layer
│       ├── proxy/                 # Proxy server, forwarder, failover
│       ├── session_manager/       # Session history scanning
│       ├── codex_history_migration/
│       ├── deeplink/              # Deep link import
│       └── platform.rs            # Platform trait abstraction
├── src-tauri/                     # Desktop shell (Tauri 2)
│   └── src/                       # Thin wrappers calling core
├── src-web/                       # Headless web server
│   └── src/
│       ├── main.rs                # Entry point (axum + graceful shutdown)
│       ├── routes.rs              # HTTP routes + 265 command dispatchers
│       └── platform_web.rs        # HeadlessPlatform impl
├── src/                           # Frontend (React + TypeScript)
│   ├── web/shims/                 # Tauri API web adapters
│   │   ├── core.ts                # invoke() → fetch /api/invoke
│   │   ├── event.ts               # listen() → EventSource
│   │   ├── plugin-dialog.ts        # HTML <input type="file">
│   │   ├── app.ts                 # getVersion() → /api/version
│   │   ├── window.ts              # no-op stub
│   │   └── path.ts                # homeDir/join → /api/info
│   └── ...                        # Same React components as desktop
├── vite.web.config.ts             # Web frontend build config
├── dist-web/                      # Built frontend (served by cc-switch-web)
└── _dev/                          # Architecture docs & migration roadmap
```

### Key Design: Platform Trait

The `Platform` trait in `cc-switch-core` abstracts all platform-specific operations:

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

- **Desktop**: `TauriPlatform` implements this via `AppHandle::emit()`, `tauri_plugin_dialog`, etc.
- **Web**: `HeadlessPlatform` implements this via SSE broadcast, `arboard` clipboard, no-op window operations.

Commands depend on `&dyn Platform`, not on Tauri types. This is why the same command works in both modes.

## FAQ

<details>
<summary><strong>Can I use both desktop and web versions on the same machine?</strong></summary>

Yes. Both versions share the same `~/.cc-switch/cc-switch.db` SQLite database and the same `~/.claude/`, `~/.codex/`, `~/.gemini/` config files. You can use the desktop app when you have a GUI, and the web server when you're SSH'd into the same machine.

</details>

<details>
<summary><strong>What's missing compared to the desktop version?</strong></summary>

Only 5 commands are permanently unavailable in web mode:
- `enter_lightweight_mode` / `exit_lightweight_mode` — Desktop "mini mode" concept
- `install_update_and_restart` — Use systemd/docker to update instead
- `launch_hermes_dashboard` — Opens a system terminal (no terminal on headless server)
- `set_window_theme` — No-op (CSS handles theming in browser)

Everything else works identically.

</details>

<details>
<summary><strong>How do I change the listen port?</strong></summary>

Currently the port is hardcoded to `18180` in `src-web/src/main.rs`. To change it, modify:

```rust
let listener = tokio::net::TcpListener::bind("127.0.0.1:18180")
```

Future versions will support environment variable configuration (`CC_SWITCH_WEB_PORT`).

</details>

<details>
<summary><strong>Is it secure? Can I expose it to the network?</strong></summary>

By default, the server binds to `127.0.0.1` (localhost only). To expose it:

1. Change bind address to `0.0.0.0:18180` in `main.rs`
2. Use a reverse proxy (Nginx/Caddy) with TLS + authentication
3. Consider adding token-based auth (planned feature)

**Do not** expose the raw server to the internet without a reverse proxy.

</details>

<details>
<summary><strong>How does the file dialog work in a browser?</strong></summary>

The frontend shim (`src/web/shims/plugin-dialog.ts`) intercepts `open_file_dialog` and `save_file_dialog` commands. When you select a file, the shim:

1. Creates an `<input type="file">` element
2. User picks a file in the browser's native file picker
3. The shim uploads the file to `POST /api/upload`
4. The server saves it to `/tmp/cc-switch-web-uploads/` and returns a path
5. The path is passed to subsequent commands (e.g., `import_config_from_file`)

This is transparent to the frontend business code — it just calls `invoke("open_file_dialog")` and gets a path string back, same as Tauri.

</details>

## Contributing

Issues and suggestions are welcome!

Before submitting PRs, please ensure:

```bash
# Core tests pass
cargo test -p cc-switch-core --all-targets

# Web crate compiles
cargo check -p cc-switch-web

# Frontend builds
npx pnpm@10 build:web

# Zero Tauri/GTK dependencies in web crate
cargo tree -p cc-switch-web -i tauri 2>&1 | grep "did not match"
cargo tree -p cc-switch-web -i gtk 2>&1 | grep "did not match"
```

## License

MIT © Jason Young (original CC Switch) · Web adaptation by David Niteas

## Acknowledgments

- [CC Switch](README_original.md) by [Jason Young](https://github.com/farion1231) — The original desktop application
- [Axum](https://github.com/tokio-rs/axum) — The web framework
- [Tauri](https://tauri.app/) — The desktop framework (still used for the desktop version)
