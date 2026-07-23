<div align="center">

# CC Switch Web

### CC Switch のヘッドレス Web サーバー —— どんなサーバーでも AI コーディングツールを管理

[![Version](https://img.shields.io/badge/version-3.17.0-blue.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Docker%20%7C%20Rust%20%E3%81%AE%E3%81%82%E3%82%8B%E5%85%A8OS-lightgrey.svg)](https://github.com/DavidNiteas/cc-switch-web)
[![Built with Axum](https://img.shields.io/badge/built%20with-Axum%200.7-blue.svg)](https://github.com/tokio-rs/axum)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**[CC Switch](README_original.md) のヘッドレス版** —— デスクトップ GUI のないヘッドレス Linux サーバー、クラウド VM、Docker コンテナなどでネイティブに動作します。

[English](README_WEB.md) | [中文](README_WEB_ZH.md) | 日本語 | [オリジナル CC Switch README](README_original.md) | [変更履歴](CHANGELOG.md)

</div>

---

## CC Switch Web が必要な理由

[CC Switch](README_original.md) は、GUI で AI コーディングツール（Claude Code、Codex、Gemini CLI など）を管理するデスクトップアプリです。しかし、**ヘッドレスサーバー** —— クラウド VM、Docker コンテナ、CI ランナー —— でデスクトップ環境、GTK、ディスプレイがない場合は？

**CC Switch Web** がその答えです：

- **ゼロ GUI 依存** —— GTK、WebKitGTK、X11 不要。純粋な Rust + Axum HTTP サーバー。任意の Linux サーバーで動作。
- **同一ビジネスロジック** —— デスクトップ版と同じ `cc-switch-core` クレートを共有。プロバイダー管理、プロキシ、MCP、プロンプト、スキル、使用統計 —— 全て同一。
- **ブラウザベース UI** —— 同じ React フロントエンドを単一の実行ファイルに埋め込み、内蔵 HTTP サーバーが提供。ブラウザで `http://localhost:18180` を開くだけ。
- **100% コマンド同等** —— 全 265 の Tauri コマンドに Web 版が存在：251 実装、4 フロントエンド shim（ファイルダイアログ）、1 リスタート、5 永続フォールバック、2 no-op、2 部分移行。

## クイックスタート

### ビルド済みバイナリをダウンロード

```bash
chmod +x cc-switch-web
RUST_LOG=info ./cc-switch-web
# ブラウザで http://127.0.0.1:18180 を開く
```

### ソースからビルド

```bash
# 前提条件: Rust 1.85+、Node.js 18+、pnpm 10+

# 1. フロントエンドビルド
npx pnpm@10 install
npx pnpm@10 build:web

# 2. バックエンドビルド（dist-web/ を実行ファイルに埋め込む）
cargo build --release --bin cc-switch-web

# 3. 実行
RUST_LOG=info ./target/release/cc-switch-web
```

### 初回起動

```bash
# サーバー起動（初回起動時に ~/.cc-switch/cc-switch.db を作成）
RUST_LOG=info ./cc-switch-web

# ブラウザで http://127.0.0.1:18180 を開く

# 動作確認
curl http://127.0.0.1:18180/api/version
# {"version":"3.17.0"}
```

## API リファレンス

| エンドポイント | メソッド | 説明 |
|----------------|----------|------|
| `/api/invoke` | POST | 汎用コマンドディスパッチャー（265 コマンド） |
| `/api/version` | GET | サーバーバージョン取得 |
| `/api/info` | GET | サーバー情報取得 |
| `/api/upload` | POST | ファイルアップロード（multipart） |
| `/api/download/:filename` | GET | サーバー側ファイルダウンロード |
| `/api/restart` | POST | グレースフルシャットダウン + systemd 再起動 |

### コマンド呼び出し

```bash
# 設定取得
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"get_settings"}'

# プロバイダー取得
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"get_providers","args":{"app":"claude"}}'

# プロキシサーバー起動
curl -X POST http://127.0.0.1:18180/api/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"start_proxy_server"}'
```

## デプロイ

### systemd サービス

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

[Install]
WantedBy=multi-user.target
```

## デスクトップ版との違い

| 機能 | デスクトップ (Tauri) | Web |
|------|---------------------|-----|
| プロバイダー管理 | ✅ | ✅ |
| プロキシ & フェイルオーバー | ✅ | ✅ |
| MCP、プロンプト、スキル | ✅ | ✅ |
| 使用量 & コスト追跡 | ✅ | ✅ |
| セッションマネージャー | ✅ | ✅ |
| Copilot OAuth | ✅ | ✅ |
| ファイルダイアログ | ネイティブ OS ダイアログ | ブラウザ `<input type="file">` |
| システムトレイ | ✅ | ❌（GUI なし） |
| 自動アップデーター | Tauri updater | GitHub releases チェック |
| アプリ再起動 | `app.restart()` | `/api/restart` + systemd |
| ライトモード | ✅ | ❌（デスクトップ概念） |
| フォルダ/ターミナルを開く | `opener` / `launch_terminal` | パス/コマンド文字列を返す |
| テーマ | ウィンドウテーマ API | No-op（CSS が処理） |

**変更なし**: 全データは同じ `~/.cc-switch/cc-switch.db` SQLite データベースに保存。

## 開発

### 前提条件

- Rust 1.85+
- Node.js 18+
- pnpm 10+

### 開発ワークフロー

```bash
# フロントエンド依存関係インストール
npx pnpm@10 install

# フロントエンドビルド（Web モード）
npx pnpm@10 build:web

# バックエンドビルド & 実行
cargo build --bin cc-switch-web
RUST_LOG=info ./target/debug/cc-switch-web

# ブラウザで http://127.0.0.1:18180 を開く
```

### テスト

```bash
# Core 単体テスト（1836 テスト）
cargo test -p cc-switch-core --all-targets

# Web クレートテスト
cargo test -p cc-switch-web --all-targets

# ゼロ Tauri/GTK 依存の確認
cargo tree -p cc-switch-web -i tauri  # "did not match" が出力されるべき
cargo tree -p cc-switch-web -i gtk    # "did not match" が出力されるべき

# フロントエンドビルド
npx pnpm@10 build:web
```

### プロジェクト構造

```
cc-switch-web/
├── Cargo.toml                    # Workspace ルート
├── src-core/                     # 共有ビジネスコア（Tauri/GTK 無し）
│   └── src/
│       ├── commands/              # 265 コマンド実装
│       ├── services/              # ビジネスロジック
│       ├── database/              # SQLite DAO レイヤー
│       ├── proxy/                 # プロキシサーバー、フォワーダー
│       ├── session_manager/       # セッション履歴スキャン
│       └── platform.rs            # Platform trait 抽象
├── src-tauri/                     # デスクトップシェル（Tauri 2）
├── src-web/                       # ヘッドレス Web サーバー
│   └── src/
│       ├── main.rs                # エントリポイント
│       ├── routes.rs              # HTTP ルート + コマンドディスパッチャー
│       └── platform_web.rs        # HeadlessPlatform 実装
├── src/                           # フロントエンド（React + TypeScript）
│   ├── web/shims/                 # Tauri API Web アダプター
│   └── ...
├── vite.web.config.ts             # Web フロントエンドビルド設定
├── dist-web/                      # ビルド時に cc-switch-web へ埋め込むフロントエンド
└── _dev/                          # アーキテクチャドキュメント
```

## FAQ

<details>
<summary><strong>デスクトップ版と Web 版を同じマシンで使えますか？</strong></summary>

はい。両バージョンは同じ `~/.cc-switch/cc-switch.db` SQLite データベースと同じ `~/.claude/`、`~/.codex/`、`~/.gemini/` 設定ファイルを共有します。GUI がある時はデスクトップアプリを、SSH 接続時は Web サーバーを使えます。

</details>

<details>
<summary><strong>デスクトップ版に比べて何が欠けていますか？</strong></summary>

Web モードで永続的に利用できないコマンドは 5 つだけです：
- `enter_lightweight_mode` / `exit_lightweight_mode` — デスクトップ「ミニモード」概念
- `install_update_and_restart` — systemd/docker で更新してください
- `launch_hermes_dashboard` — システムターミナルを開く（ヘッドレスサーバーにターミナルなし）
- `set_window_theme` — No-op（ブラウザでは CSS がテーマを処理）

それ以外は全て同じように動作します。

</details>

## ライセンス

MIT © Jason Young（オリジナル CC Switch）· Web 適応 by David Niteas

## 謝辞

- [CC Switch](README_original.md) by [Jason Young](https://github.com/farion1231) — オリジナルデスクトップアプリ
- [Axum](https://github.com/tokio-rs/axum) — Web フレームワーク
- [Tauri](https://tauri.app/) — デスクトップフレームワーク
