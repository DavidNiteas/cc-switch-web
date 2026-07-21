# 实验性改动回退记录（修订版）

> 更新时间：2026-07-20  
> 本版本记录了回退后的新方向，以及原实验代码的状态。

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
- `src-tauri` 只做最小可见性改动，保持"非入侵"。
- 借鉴 `mzparse` 的 feature 分层思想，让 Web 构建能够不依赖 `wry`/GTK 编译。

## 回退后的方向修正

经过 `_dev/verify-tauri-compile/` 最小 crate 验证，发现 **Tauri 2.11.x 在 Linux 上无法通过 feature 关闭 GTK 依赖**：

- `tauri` crate 在 Linux 目标下直接依赖 `gtk`（非 optional）；
- `tauri-runtime` crate 在 Linux 目标下直接依赖 `gtk` 和 `webkit2gtk`（均非 optional）；
- `muda` crate 在 Linux 目标下被 tauri 强制引入，且 feature 含 `gtk`。

因此，"关闭 `wry` feature 即可去掉 GTK"的路径走不通。**只要 `tauri` crate 还在依赖图里，无头 Linux 就无法干净编译**。

基于这一实验结果，项目方向进一步修正为 **"Core + Tauri + Web 三层分离"**：

- `cc-switch-core`：纯 Rust 业务核心，无 Tauri/GTK 依赖；
- `cc-switch-tauri`：桌面薄壳，依赖 core + Tauri；
- `cc-switch-web`：无头 Web 服务，依赖 core + axum。

所有原生命令中直接调用的 Tauri API，通过 `Platform` trait 抽象。命令业务实现下移到 core，Tauri/axum 只保留薄壳。

## 实验代码的现状

原 `_dev/experimental-headless.patch` 中的代码：

- **仍可运行**：作为功能原型，MockRuntime + axum 桥接是可行的；
- **不能作为最终方案**：因为它仍依赖 `tauri` crate，编译期需要 GTK  workaround；
- **其中的前端 shim 和 vite.web.config.ts 设计可回收**：新的 `cc-switch-web` 可以继续使用 `src/web/shims/*.ts` 作为前端适配层。

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

注意恢复后仍受本机 GTK 环境限制，需要 `.gtk-stub/` 或重新生成。在新的三层分离架构完成后，此 patch 将被废弃。
