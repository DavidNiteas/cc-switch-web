<div align="center">

# CC Switch Web

### Headless Web Server for CC Switch — Manage AI Coding Tools on Any Server

[![Version](https://img.shields.io/badge/version-3.17.0-blue.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Docker%20%7C%20any%20OS%20with%20Rust-lightgrey.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Built with Axum](https://img.shields.io/badge/built%20with-Axum%200.7-blue.svg)](https://github.com/tokio-rs/axum)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**The headless web server for [CC Switch](README_original.md)** — runs natively on headless Linux servers, cloud VMs, Docker containers, and any environment without a desktop GUI.

English | [中文](README_ZH.md) | [日本語](README_JA.md) | [Full Documentation](README_WEB.md) | [Desktop CC Switch (original)](README_original.md) | [Changelog](CHANGELOG.md)

</div>

---

## Quick Start

```bash
# Download the binary, run it, open http://127.0.0.1:18180
RUST_LOG=info ./cc-switch-web
```

## What is CC Switch Web?

CC Switch Web is the **headless edition** of the [CC Switch](README_original.md) desktop application. It provides the same business logic — provider management, proxy, MCP, prompts, skills, usage tracking — served over HTTP instead of a desktop GUI.

- **No desktop environment required** — pure Rust + Axum, runs on any Linux server
- **Same React frontend** — served by the built-in HTTP server, open in any browser
- **100% command parity** — all 265 commands work (with minor differences for file dialogs, tray, etc.)

## Full Documentation

➡️ **[README_WEB.md](README_WEB.md)** — Architecture, API reference, deployment guide, development guide, FAQ, and more.

## Original Desktop Version

The original CC Switch is a **cross-platform desktop app** (Windows, macOS, Linux) built with Tauri 2. It provides a native GUI with system tray integration, window management, and auto-updates.

➡️ **[README_original.md](README_original.md)** — Original CC Switch documentation (desktop app).

## License

MIT © Jason Young (original CC Switch) · Web adaptation by David Niteas