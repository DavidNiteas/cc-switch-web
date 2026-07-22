<div align="center">

# CC Switch Web

### CC Switch のヘッドレス Web サーバー —— どんなサーバーでも AI コーディングツールを管理

[![Version](https://img.shields.io/badge/version-3.17.0-blue.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Docker%20%7C%20Rust%20%E3%81%AE%E3%81%82%E3%82%8B%E5%85%A8OS-lightgrey.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Built with Axum](https://img.shields.io/badge/built%20with-Axum%200.7-blue.svg)](https://github.com/tokio-rs/axum)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**[CC Switch](README_original.md) のヘッドレス Web サーバー** —— デスクトップ GUI のないヘッドレス Linux サーバー、クラウド VM、Docker コンテナなどでネイティブに動作します。

[English](README.md) | [中文](README_ZH.md) | 日本語 | [完全なドキュメント](README_WEB_JA.md) | [デスクトップ版 CC Switch（オリジナル）](README_original.md) | [変更履歴](CHANGELOG.md)

</div>

---

## クイックスタート

```bash
# バイナリをダウンロード、実行、http://127.0.0.1:18180 を開く
RUST_LOG=info ./cc-switch-web
```

## CC Switch Web とは？

CC Switch Web は、[CC Switch](README_original.md) デスクトップアプリケーションの**ヘッドレス版**です。プロバイダー管理、プロキシ、MCP、プロンプト、スキル、使用量追跡など、同じビジネスロジックを HTTP 経由で提供します。

- **デスクトップ環境不要** — 純粋な Rust + Axum、任意の Linux サーバーで動作
- **同じ React フロントエンド** — 内蔵 HTTP サーバーが提供、ブラウザで開くだけ
- **100% コマンド同等** — 全 265 コマンドが使用可能（ファイルダイアログ、トレイなどに若干の違いあり）

## 完全なドキュメント

➡️ **[README_WEB_JA.md](README_WEB_JA.md)** — アーキテクチャ、API リファレンス、デプロイガイド、開発ガイド、FAQ など。

## デスクトップオリジナル版

オリジナルの CC Switch は、Tauri 2 で構築された**クロスプラットフォームデスクトップアプリ**（Windows、macOS、Linux）です。ネイティブ GUI、システムトレイ統合、ウィンドウ管理、自動更新を提供します。

➡️ **[README_original.md](README_original.md)** — オリジナル CC Switch ドキュメント（デスクトップアプリ）。

## ライセンス

MIT © Jason Young（オリジナル CC Switch）· Web 適応 by David Niteas