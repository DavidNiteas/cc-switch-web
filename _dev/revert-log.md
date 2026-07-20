# 实验性改动回退记录

## 本次实验做了什么

在 `/home/daiql/cc-switch` 上实现了一个可用的 headless web 原型，验证 MockRuntime + axum 桥接方案可行。

### 改动的 tracked 文件

| 文件 | 改动内容 |
|------|----------|
| `package.json` | 增加 `build:web` 脚本 |
| `pnpm-workspace.yaml` | 修复 pnpm 11 `allowBuilds` 占位自写回问题 |
| `src-tauri/Cargo.toml` | 给 tauri 增加 `test` feature |
| `src-tauri/src/app_store.rs` | 4 个函数泛型化 `<R: tauri::Runtime>` |
| `src-tauri/src/lib.rs` | 抽出命令清单宏、新增 `mod headless;`、改动 setup 结构 |
| `src-tauri/src/usage_events.rs` | `init()` 泛型化，内部存 emit 闭包 |

### 新增文件/目录

- `src-tauri/src/headless.rs` — MockRuntime + axum 服务
- `src-tauri/src/bin/headless.rs` — headless 二进制入口
- `src/web/shims/` — 8 个 Tauri API shim
- `vite.web.config.ts` — Web 构建配置
- `.gtk-stub/` — 本机 GTK dev 包缺失时的编译期 workaround
- `dist-web/` — Web 前端构建产物

### 验证结果

- `cargo build --bin headless` 通过
- `cargo check --all-targets` 0 warning
- curl 验证核心命令读写正常
- Playwright 验证主界面完整渲染
- 服务跑在 `http://127.0.0.1:18180`

完整改动 patch 已保存为 `_dev/experimental-headless.patch`，必要时可恢复。

## 为什么回退

用户决定采用更彻底的长期架构：

- 不再把 headless 代码塞进 `src-tauri`，而是新建独立 `src-web` crate。
- `src-tauri` 只做最小可见性/feature 改动，保持"非入侵"。
- 借鉴 `mzparse` 的 feature 分层思想，让 Web 构建能够不依赖 `wry`/GTK 编译。

因此当前实验性实现被整体撤销，思路和方法已沉淀到 `_dev/` 文档中。

## 回退后的状态

```bash
git status --short   # 应为空
git status           # working tree clean
```

## 恢复实验代码的方法（仅供临时参考）

```bash
cd /home/daiql/cc-switch
git apply _dev/experimental-headless.patch
pnpm install
pnpm build:web
cd src-tauri
cargo build --bin headless
```

注意恢复后仍受本机 GTK 环境限制，需要 `.gtk-stub/` 或重新生成。
