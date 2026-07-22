<div align="center">

# CC Switch Web

### CC Switch 的无头 Web 服务器 —— 在任何服务器上管理 AI 编码工具

[![Version](https://img.shields.io/badge/version-3.17.0-blue.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Docker%20%7C%20任何有%20Rust%20的%20OS-lightgrey.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Built with Axum](https://img.shields.io/badge/built%20with-Axum%200.7-blue.svg)](https://github.com/tokio-rs/axum)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**[CC Switch](README_original.md) 的无头 Web 服务器** —— 原生运行在无头 Linux 服务器、云虚拟机、Docker 容器以及任何没有桌面 GUI 的环境中。

[English](README.md) | 中文 | [日本語](README_JA.md) | [完整文档](README_WEB_ZH.md) | [桌面版 CC Switch（原版）](README_original.md) | [更新日志](CHANGELOG.md)

</div>

---

## 快速开始

```bash
# 下载二进制文件，运行，打开 http://127.0.0.1:18180
RUST_LOG=info ./cc-switch-web
```

## 什么是 CC Switch Web？

CC Switch Web 是 [CC Switch](README_original.md) 桌面应用的**无头版本**。它提供相同的业务逻辑——供应商管理、代理、MCP、提示词、技能、用量统计——通过 HTTP 提供，而非桌面 GUI。

- **无需桌面环境** — 纯 Rust + Axum，可在任何 Linux 服务器上运行
- **相同的 React 前端** — 由内置 HTTP 服务器提供，在任意浏览器中打开
- **100% 命令对等** — 全部 265 个命令均可使用（文件对话框、托盘等有细微差异）

## 完整文档

➡️ **[README_WEB_ZH.md](README_WEB_ZH.md)** — 架构、API 参考、部署指南、开发指南、FAQ 等。

## 桌面原版

原版 CC Switch 是一款**跨平台桌面应用**（Windows、macOS、Linux），基于 Tauri 2 构建。提供原生 GUI、系统托盘集成、窗口管理和自动更新。

➡️ **[README_original.md](README_original.md)** — 原版 CC Switch 文档（桌面应用）。

## 许可证

MIT © Jason Young（原始 CC Switch）· Web 适配由 David Niteas 完成