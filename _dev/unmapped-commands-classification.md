# 未覆盖命令真实阻塞原因分类（阶段五收尾后）

> 目标：重新审视 `migration-roadmap.md` 附录 D 中 176 个"未覆盖"命令，按真实阻塞原因重新分类，验证"除系统 GUI 组件外理论上均可迁移"的假设。
> 审计时间：2026-07-20
> 审计范围：`src-tauri/src/commands/**/*.rs` 中附录 D 列出的 176 个未暴露命令
> 验证状态：`cargo check -p cc-switch-core` 与 `cargo check -p cc-switch-web` 均通过

## 1. 核心结论

**用户的判断基本正确：176 个未覆盖命令中，真正因"系统 GUI/桌面集成"而无法迁移的只有 19 个（10.8%）。**

| 分类 | 数量 | 占比 | 说明 |
|------|------|------|------|
| **A. 可直接迁移** | 112 | 63.6% | 只依赖 core `Database` / `Service`，无 GUI/系统调用 |
| **B. 需把 tauri 模块下沉到 core** | 38 | 21.6% | 依赖的 service/module（usage_stats、session_manager、codex_history_migration、OAuth AuthState 等）尚未完全下沉 |
| **C. 需 Platform 抽象或事件拆分** | 2 | 1.1% | 业务逻辑可迁移，但 UI 副作用（事件发射、托盘刷新）需拆分到 tauri 外壳 |
| **D. 系统 GUI/桌面集成（真正无法迁移）** | 19 | 10.8% | 窗口、托盘、文件对话框、系统浏览器、系统终端、进程重启、更新器、deep-link 注册等 |
| **E. 应用生态/外部集成（技术上可迁移，Web 价值需评估）** | 5 | 2.8% | 依赖本地 PATH/环境变量/外部下载，Web 环境可能无对应工具或存在安全风险 |
| **合计** | **176** | **100%** | — |

**理论上可迁移（A+B+C）合计 152 个，占未覆盖命令的 86.4%。**

> **修正说明**：经代码审计，`会话/用量`组中的 11 个用量统计命令依赖的 `services/usage_stats` 仍在 `src-tauri`，并非已在 core `Database` 中。因此这 11 个命令从 A 类修正为 B 类。A 类总数相应从 123 调整为 112，B 类从 27 调整为 38。

这意味着：
- 之前附录 D 中"暂时无法覆盖"的口径偏悲观，大量命令只是因为改造进度未到，而非真的无法迁移。
- 下一阶段如果把 **A 类命令封装**和 **B 类模块下沉**做完，Web 覆盖率可从 33.6% 提升到 **约 85-90%**。
- 最终只剩 **19 个 D 类命令**需要在 Web 端返回明确错误并由前端兜底。

---

## 2. 分类标准

| 分类 | 标准 | 改造方式 |
|------|------|---------|
| **A** | 命令只操作 `Database`、core `Service` 或文件系统，无 `AppHandle`/`Window`/`Dialog`/`Opener`/`Tray`/`Process` 调用 | 直接在 `src-core/src/commands/` 封装，Web 路由注册即可 |
| **B** | 命令依赖的 service/module 仍在 `src-tauri`（如 `session_manager`、`codex_history_migration`、`CopilotAuthState`、`StreamCheckService` 探测逻辑） | 先把对应 service/module 迁移/下沉到 core，再封装命令 |
| **C** | 命令核心业务可迁移，但包含 `emit_event`、`tray refresh`、打开浏览器等 UI 副作用 | core 命令返回结构化结果，由 tauri/web 外壳各自处理副作用 |
| **D** | 命令本身就是 GUI/系统功能：窗口控制、托盘、文件对话框、系统浏览器、系统终端、进程重启、更新器、deep-link 注册、 lightweight 模式 | 不迁移业务逻辑；Web 端返回明确 "not supported" 错误 |
| **E** | 命令依赖本地应用生态（CLI 工具、环境变量、GitHub 下载），技术上可迁移但 Web 环境可能无意义或存在安全边界 | 先评估 Web 暴露价值；若需要，增加权限/沙箱控制后再迁移 |

---

## 3. 逐组重新分类

### 3.1 Config 其他（10 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `extract_common_config_snippet` | 需文件对话框/TOML 编辑 | **A** | 调用 `ProviderService`，无 GUI 依赖 |
| `get_claude_common_config_snippet` | 需文件对话框/TOML 编辑 | **A** | 纯 `db.get_config_snippet` |
| `get_claude_config_status` | 需文件对话框/TOML 编辑 | **A** | 调用 `config::get_claude_config_status()`，仅文件系统检查 |
| `get_common_config_snippet` | 需文件对话框/TOML 编辑 | **A** | 纯 `db.get_config_snippet` |
| `open_app_config_folder` | 需文件对话框/TOML 编辑 | **D** | 使用 `opener` 打开系统文件夹 |
| `open_config_folder` | 需文件对话框/TOML 编辑 | **D** | 使用 `opener` 打开系统文件夹 |
| `pick_directory` | 需文件对话框/TOML 编辑 | **D** | 使用 `tauri_plugin_dialog` 文件选择器 |
| `set_claude_common_config_snippet` | 需文件对话框/TOML 编辑 | **A** | 纯 `db.set_config_snippet` |
| `set_common_config_snippet` | 需文件对话框/TOML 编辑 | **A** | `db` 写入 + `ProviderService` 同步 |
| `update_toml_common_config_snippet` | 需文件对话框/TOML 编辑 | **A** | 调用 `ProviderService` 的 TOML 编辑函数 |

**小结：A=7，D=3。** 之前的"需文件对话框"分类不准确；只有 3 个真正需要对话框。

---

### 3.2 Global Proxy 其他（1 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `scan_local_proxies` | 本地代理扫描 | **A** | TCP 扫描 `127.0.0.1` 常见端口，无 GUI 依赖 |

---

### 3.3 MCP 其他（3 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `delete_mcp_server_in_config` | 未迁移逻辑 | **A** | 调用 core `McpService::delete_server` |
| `import_mcp_from_apps` | 未迁移逻辑 | **A** | 调用 core `McpService::import_from_all_apps` |
| `upsert_mcp_server_in_config` | 未迁移逻辑 | **A** | 调用 core `McpService::upsert_server` |

---

### 3.4 Misc 其他（5 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `get_tool_versions` | 终端/工具生命周期 | **E** | 探测本地 CLI `--version`，依赖 PATH/Shell；技术上可迁移，但 Web 环境通常无这些工具 |
| `open_provider_terminal` | 终端/工具生命周期 | **D** | 打开系统终端 |
| `probe_tool_installations` | 终端/工具生命周期 | **E** | 同 `get_tool_versions` |
| `run_tool_lifecycle_action` | 终端/工具生命周期 | **E** | 安装/更新工具，需要 shell/管理员权限和外部下载 |
| `set_window_theme` | 终端/工具生命周期 | **D** | GUI 主题 |

**小结：D=2，E=3。** 工具生命周期命令不是 GUI，但属于本地应用生态；Web 暴露需谨慎。

---

### 3.5 OAuth/认证桌面集成（8 个）

> 注：`auth_*.rs` 中的新版统一认证命令未在附录 D 列出，但 `copilot_*.rs` 旧命令仍在。核心 OAuth 逻辑（`proxy/providers/copilot_auth.rs`）已在 core，只是 `CopilotAuthState` 管理仍在 tauri。

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `copilot_start_device_flow` | OAuth 桌面集成 | **B** | 设备流启动逻辑在 core，但 AuthState 需下沉 |
| `copilot_poll_for_auth` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_poll_for_account` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_list_accounts` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_remove_account` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_set_default_account` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_get_auth_status` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_is_authenticated` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_logout` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_get_token` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_get_token_for_account` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_get_models` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_get_models_for_account` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_get_usage` | OAuth 桌面集成 | **B** | 同上 |
| `copilot_get_usage_for_account` | OAuth 桌面集成 | **B** | 同上 |

附录 D 只列出 8 个，全部为 **B**。

---

### 3.6 Provider/Claude Desktop 其他（11 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `add_provider` | Claude Desktop 集成 | **A** | `ProviderService::add` |
| `ensure_claude_desktop_official_provider` | Claude Desktop 集成 | **A** | 纯 `db.ensure_official_seed_by_id` |
| `ensure_codex_official_provider` | Claude Desktop 集成 | **A** | 纯 `db.ensure_official_seed_by_id` |
| `get_claude_desktop_default_routes` | Claude Desktop 集成 | **A** | 纯函数 |
| `get_claude_desktop_status` | Claude Desktop 集成 | **A** | `db` + `proxy_service.is_running()` |
| `import_claude_desktop_providers_from_claude` | Claude Desktop 集成 | **A** | 纯 `db` 操作 |
| `queryProviderUsage` | Claude Desktop 集成 | **C** | 核心查询可迁移，但需发射 `usage-cache-updated` 事件并刷新托盘 |
| `read_live_provider_settings` | Claude Desktop 集成 | **A** | 读取应用配置文件 |
| `remove_provider_from_live_config` | Claude Desktop 集成 | **A** | `ProviderService::remove_from_live_config` |
| `testUsageScript` | Claude Desktop 集成 | **A** | `ProviderService` 测试 usage script |
| `update_provider` | Claude Desktop 集成 | **A** | `ProviderService::update` |

**小结：A=10，C=1。** Claude Desktop 相关命令绝大多数是纯数据操作。

---

### 3.7 Provider/Profile 其他（1 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `list_profiles` | provider/profile 其他 | **A** | `ProfileService::list_profiles` |

---

### 3.8 Settings/GUI/系统（17 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `check_app_update_available` | GUI/系统 | **D** | Tauri updater |
| `get_app_config_dir_override` | GUI/系统 | **A** | core `app_store` 内存缓存 |
| `get_auto_launch_status` | GUI/系统 | **B** | `auto-launch` crate 已在 core deps，但命令层未下沉 |
| `get_copilot_optimizer_config` | GUI/系统 | **A** | 纯 `db` 查询 |
| `get_log_config` | GUI/系统 | **A** | 纯 `db` 查询 |
| `get_optimizer_config` | GUI/系统 | **A** | 纯 `db` 查询 |
| `get_rectifier_config` | GUI/系统 | **A** | 纯 `db` 查询 |
| `has_codex_unify_history_backup` | GUI/系统 | **B** | 依赖 `codex_history_migration` 模块，仍在 tauri |
| `install_update_and_restart` | GUI/系统 | **D** | Tauri updater + 进程重启 |
| `restart_app` | GUI/系统 | **D** | 进程重启 |
| `restore_codex_unified_history` | GUI/系统 | **B** | 依赖 `codex_history_migration` 模块 |
| `set_app_config_dir_override` | GUI/系统 | **A** | core `app_store` 内存缓存 |
| `set_auto_launch` | GUI/系统 | **B** | `auto-launch` crate |
| `set_copilot_optimizer_config` | GUI/系统 | **A** | 纯 `db` 写入 |
| `set_log_config` | GUI/系统 | **A** | 纯 `db` 写入 |
| `set_optimizer_config` | GUI/系统 | **A** | 纯 `db` 写入 |
| `set_rectifier_config` | GUI/系统 | **A** | 纯 `db` 写入 |

**小结：A=10，B=4，D=3。** Settings 中大量被标为"GUI/系统"的命令其实是纯 DB 配置读写。

---

### 3.9 Skill 网络/发现/旧 API（12 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `check_skill_updates` | 网络/发现/旧 API | **B** | 依赖 `SkillService` 网络方法，需下沉到 core |
| `discover_available_skills` | 网络/发现/旧 API | **B** | 同上 |
| `get_skills` | 网络/发现/旧 API | **A** | 兼容旧 API，核心是 `db` 查询 |
| `get_skills_for_app` | 网络/发现/旧 API | **A** | 同上 |
| `install_skill` | 网络/发现/旧 API | **B** | 依赖 `SkillService::install`，需下沉 |
| `install_skill_for_app` | 网络/发现/旧 API | **B** | 同上 |
| `install_skill_unified` | 网络/发现/旧 API | **B** | 同上 |
| `restore_skill_backup` | 网络/发现/旧 API | **A** | 本地备份恢复 |
| `search_skills_sh` | 网络/发现/旧 API | **B** | 依赖 skills.sh 公共 API |
| `uninstall_skill` | 网络/发现/旧 API | **A** | 本地卸载 |
| `uninstall_skill_for_app` | 网络/发现/旧 API | **A** | 同上 |
| `update_skill` | 网络/发现/旧 API | **B** | 依赖 `SkillService::update_skill`，需下沉 |

**小结：A=5，B=7。** Skill 网络/发现命令技术上可迁移，但 Web 暴露时需要评估"从互联网下载并安装可执行 skill"的安全边界。

---

### 3.10 代理高级/OAuth/证书（21 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `get_circuit_breaker_config` | 代理高级 | **A** | `proxy_service` / `db` |
| `get_circuit_breaker_stats` | 代理高级 | **A** | `proxy_service`（当前 stub） |
| `get_default_cost_multiplier` | 代理高级 | **A** | `db` 查询 |
| `get_global_proxy_config` | 代理高级 | **A** | `db` 查询 |
| `get_pricing_model_source` | 代理高级 | **A** | `db` 查询 |
| `get_provider_health` | 代理高级 | **A** | `proxy_service` |
| `get_proxy_config` | 代理高级 | **A** | `proxy_service` / `db` |
| `get_proxy_config_for_app` | 代理高级 | **A** | `proxy_service` / `db` |
| `get_proxy_takeover_status` | 代理高级 | **A** | `proxy_service` |
| `is_live_takeover_active` | 代理高级 | **A** | `proxy_service` |
| `is_proxy_running` | 代理高级 | **A** | `proxy_service` |
| `reset_circuit_breaker` | 代理高级 | **C** | 业务可迁移，但需发射 `circuit-breaker-reset` 事件 |
| `set_default_cost_multiplier` | 代理高级 | **A** | `db` 写入 |
| `set_pricing_model_source` | 代理高级 | **A** | `db` 写入 |
| `set_proxy_takeover_for_app` | 代理高级 | **A** | `proxy_service` |
| `stop_proxy_with_restore` | 代理高级 | **A** | `proxy_service` |
| `switch_proxy_provider` | 代理高级 | **A** | `proxy_service` |
| `update_circuit_breaker_config` | 代理高级 | **A** | `proxy_service` / `db` |
| `update_global_proxy_config` | 代理高级 | **A** | `db` 写入 |
| `update_proxy_config` | 代理高级 | **A** | `proxy_service` / `db` |
| `update_proxy_config_for_app` | 代理高级 | **A** | `proxy_service` / `db` |

**小结：A=20，C=1。** 代理高级命令几乎都可以直接迁移，之前被过度归类为"高级/OAuth/证书"。

---

### 3.11 会话/用量（18 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `check_provider_limits` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `delete_model_pricing` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats` 的 `ModelPricingInfo` 等类型 |
| `delete_session` | 依赖未迁移 usage_stats | **B** | `session_manager::delete_session` |
| `delete_sessions` | 依赖未迁移 usage_stats | **B** | `session_manager::delete_sessions` |
| `get_model_pricing` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `get_model_stats` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `get_provider_stats` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `get_request_detail` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `get_request_logs` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `get_session_messages` | 依赖未迁移 usage_stats | **B** | `session_manager::load_messages` |
| `get_usage_data_sources` | 依赖未迁移 usage_stats | **B** | `session_usage` service |
| `get_usage_summary` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `get_usage_summary_by_app` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `get_usage_trends` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats`，尚未迁移到 core |
| `launch_session_terminal` | 依赖未迁移 usage_stats | **D** | 打开系统终端 |
| `list_sessions` | 依赖未迁移 usage_stats | **B** | `session_manager::scan_sessions` |
| `sync_session_usage` | 依赖未迁移 usage_stats | **B** | `session_usage*` services |
| `update_model_pricing` | 依赖未迁移 usage_stats | **B** | 依赖 `services/usage_stats` 的 `ModelPricingInfo` 等类型 |

**小结：B=17，D=1。** 用量统计命令依赖的 `services/usage_stats` 和 `session_manager`/`session_usage` 服务均在 tauri，需要整体下沉到 core 后才能释放这些命令。

---

### 3.12 同步后端（10 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `s3_sync_download` | 同步后端 | **A** | `s3_sync_service::download` 已在 core |
| `s3_sync_fetch_remote_info` | 同步后端 | **A** | 同上 |
| `s3_sync_save_settings` | 同步后端 | **A** | settings 读写 |
| `s3_sync_upload` | 同步后端 | **A** | `s3_sync_service::upload` 已在 core |
| `s3_test_connection` | 同步后端 | **A** | 同上 |
| `webdav_sync_download` | 同步后端 | **A** | `webdav_sync_service::download` 已在 core |
| `webdav_sync_fetch_remote_info` | 同步后端 | **A** | 同上 |
| `webdav_sync_save_settings` | 同步后端 | **A** | settings 读写 |
| `webdav_sync_upload` | 同步后端 | **A** | `webdav_sync_service::upload` 已在 core |
| `webdav_test_connection` | 同步后端 | **A** | 同上 |

**小结：A=10。** 同步服务已完全在 core，命令层只是薄壳未写。

---

### 3.13 应用专属/环境/工作区（36 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `check_env_conflicts` | 应用专属 | **E** | 检查本地环境变量冲突，Web 环境价值有限 |
| `delete_daily_memory_file` | 应用专属 | **A** | 文件系统删除 |
| `delete_env_vars` | 应用专属 | **E** | 修改本地环境变量 |
| `get_coding_plan_quota` | 应用专属 | **A** | HTTP 查询订阅配额 |
| `get_hermes_live_provider` | 应用专属 | **A** | 读取 Hermes 配置文件 |
| `get_hermes_live_provider_ids` | 应用专属 | **A** | 同上 |
| `get_hermes_memory` | 应用专属 | **A** | 文件读取 |
| `get_hermes_memory_limits` | 应用专属 | **A** | 文件读取 |
| `get_hermes_model_config` | 应用专属 | **A** | 文件读取 |
| `get_openclaw_agents_defaults` | 应用专属 | **A** | 文件读取 |
| `get_openclaw_default_model` | 应用专属 | **A** | 文件读取 |
| `get_openclaw_env` | 应用专属 | **A** | 文件读取 |
| `get_openclaw_live_provider` | 应用专属 | **A** | 文件读取 |
| `get_openclaw_live_provider_ids` | 应用专属 | **A** | 文件读取 |
| `get_openclaw_model_catalog` | 应用专属 | **A** | 文件读取 |
| `get_openclaw_tools` | 应用专属 | **A** | 文件读取 |
| `import_hermes_providers_from_live` | 应用专属 | **A** | `ProviderService` 导入 |
| `import_openclaw_providers_from_live` | 应用专属 | **A** | `ProviderService` 导入 |
| `launch_hermes_dashboard` | 应用专属 | **D** | 打开系统终端运行 `hermes dashboard` |
| `list_daily_memory_files` | 应用专属 | **A** | 文件系统枚举 |
| `open_hermes_web_ui` | 应用专属 | **D** | 使用 `opener` 打开系统浏览器 |
| `open_workspace_directory` | 应用专属 | **D** | 使用 `opener` 打开系统文件夹 |
| `read_daily_memory_file` | 应用专属 | **A** | 文件读取 |
| `read_workspace_file` | 应用专属 | **A** | 文件读取 |
| `restore_env_backup` | 应用专属 | **E** | 恢复本地环境变量 |
| `scan_openclaw_config_health` | 应用专属 | **A** | 文件扫描 |
| `search_daily_memory_files` | 应用专属 | **A** | 文件搜索 |
| `set_hermes_memory` | 应用专属 | **A** | 文件写入 |
| `set_hermes_memory_enabled` | 应用专属 | **A** | 文件写入 |
| `set_openclaw_agents_defaults` | 应用专属 | **A** | 文件写入 |
| `set_openclaw_default_model` | 应用专属 | **A** | 文件写入 |
| `set_openclaw_env` | 应用专属 | **A** | 文件写入 |
| `set_openclaw_model_catalog` | 应用专属 | **A** | 文件写入 |
| `set_openclaw_tools` | 应用专属 | **A** | 文件写入 |
| `write_daily_memory_file` | 应用专属 | **A** | 文件写入 |
| `write_workspace_file` | 应用专属 | **A** | 文件写入 |

**小结：A=30，D=4，E=2。** Hermes/OpenClaw/Workspace 绝大多数命令只是文件读写，完全可以迁移。

---

### 3.14 插件/Deep-link/GUI（13 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `apply_claude_onboarding_skip` | 插件/Deep-link/GUI | **A** | 写 `~/.claude.json` 文件 |
| `apply_claude_plugin_config` | 插件/Deep-link/GUI | **A** | 写 Claude 配置文件 |
| `clear_claude_onboarding_skip` | 插件/Deep-link/GUI | **A** | 同上 |
| `enter_lightweight_mode` | 插件/Deep-link/GUI | **D** | 依赖 `AppHandle`，桌面轻量模式 |
| `exit_lightweight_mode` | 插件/Deep-link/GUI | **D** | 同上 |
| `get_claude_plugin_status` | 插件/Deep-link/GUI | **A** | 文件状态检查 |
| `import_from_deeplink` | 插件/Deep-link/GUI | **A** | 解析并导入，核心逻辑在 `deeplink` 模块 |
| `import_from_deeplink_unified` | 插件/Deep-link/GUI | **A** | 同上 |
| `is_claude_plugin_applied` | 插件/Deep-link/GUI | **A** | 文件状态检查 |
| `is_lightweight_mode` | 插件/Deep-link/GUI | **D** | 桌面轻量模式 |
| `merge_deeplink_config` | 插件/Deep-link/GUI | **A** | 配置合并 |
| `parse_deeplink` | 插件/Deep-link/GUI | **A** | URL 解析 |
| `read_claude_plugin_config` | 插件/Deep-link/GUI | **A** | 文件读取 |

**小结：A=10，D=3。** deep-link 导入/解析本身不是 GUI；只有 lightweight 模式是桌面概念。

---

### 3.15 文件对话框/备份管理（8 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `create_db_backup` | 文件对话框/备份管理 | **A** | 数据库文件备份 |
| `delete_db_backup` | 文件对话框/备份管理 | **A** | 备份文件删除 |
| `list_db_backups` | 文件对话框/备份管理 | **A** | 备份文件枚举 |
| `open_file_dialog` | 文件对话框/备份管理 | **D** | `tauri_plugin_dialog` 文件选择 |
| `open_zip_file_dialog` | 文件对话框/备份管理 | **D** | `tauri_plugin_dialog` 文件选择 |
| `rename_db_backup` | 文件对话框/备份管理 | **A** | 备份文件重命名 |
| `restore_db_backup` | 文件对话框/备份管理 | **A** | 数据库文件恢复 |
| `save_file_dialog` | 文件对话框/备份管理 | **D** | `tauri_plugin_dialog` 文件保存 |

**小结：A=5，D=3。** 备份管理是纯文件操作，完全可以迁移。

---

### 3.16 流检测（2 个）

| 命令 | 原分类 | 新分类 | 原因 |
|------|--------|--------|------|
| `stream_check_all_providers` | 探测服务在 tauri | **B** | `StreamCheckService` 在 core，但 Copilot base_url 解析依赖 `CopilotAuthState` |
| `stream_check_provider` | 探测服务在 tauri | **B** | 同上 |

---

## 4. 汇总统计

| 原分组 | 总数 | A | B | C | D | E |
|--------|------|---|---|---|---|---|
| Config 其他 | 10 | 7 | 0 | 0 | 3 | 0 |
| Global Proxy 其他 | 1 | 1 | 0 | 0 | 0 | 0 |
| MCP 其他 | 3 | 3 | 0 | 0 | 0 | 0 |
| Misc 其他 | 5 | 0 | 0 | 0 | 2 | 3 |
| OAuth/认证桌面集成 | 8 | 0 | 8 | 0 | 0 | 0 |
| Provider/Claude Desktop 其他 | 11 | 10 | 0 | 1 | 0 | 0 |
| Provider/Profile 其他 | 1 | 1 | 0 | 0 | 0 | 0 |
| Settings/GUI/系统 | 17 | 10 | 4 | 0 | 3 | 0 |
| Skill 网络/发现/旧 API | 12 | 5 | 7 | 0 | 0 | 0 |
| 代理高级/OAuth/证书 | 21 | 20 | 0 | 1 | 0 | 0 |
| 会话/用量 | 18 | 0 | 17 | 0 | 1 | 0 |
| 同步后端 | 10 | 10 | 0 | 0 | 0 | 0 |
| 应用专属/环境/工作区 | 36 | 30 | 0 | 0 | 4 | 2 |
| 插件/Deep-link/GUI | 13 | 10 | 0 | 0 | 3 | 0 |
| 文件对话框/备份管理 | 8 | 5 | 0 | 0 | 3 | 0 |
| 流检测 | 2 | 0 | 2 | 0 | 0 | 0 |
| **合计** | **176** | **112** | **38** | **2** | **19** | **5** |
| **占比** | 100% | 63.6% | 21.6% | 1.1% | 10.8% | 2.8% |

---

## 5. 迁移优先级建议

### P0：A 类命令直接迁移（123 个）

**这是覆盖率提升最快的部分。** 这些命令只需要：
1. 在 `src-core/src/commands/` 中新增/扩展对应模块（如 `usage.rs`、`proxy.rs`、`config.rs`）。
2. 在 `src-web/src/routes.rs` 注册 handler。
3. curl 验证。

推荐按文件分批：
- `proxy.rs` 高级配置（20 个 A + 1 个 C）✅ 已完成
- `s3_sync.rs` / `webdav_sync.rs`（10 个 A）✅ 已完成
- `config.rs` TOML/片段编辑（7 个 A）
- `hermes.rs` / `openclaw.rs` / `workspace.rs` 文件读写（30 个 A）
- `provider.rs` / `profile.rs` 剩余命令（11 个 A + 1 个 C）
- `settings.rs` 配置读写（10 个 A）
- `plugin.rs` / `deeplink.rs` 文件操作（10 个 A）
- `import_export.rs` 备份管理（5 个 A）
- `mcp.rs` 兼容命令（3 个 A）
- `global_proxy.rs` `scan_local_proxies`（1 个 A）
- `skill.rs` 本地兼容命令（5 个 A）

> 注：`usage.rs` 用量统计命令依赖的 `services/usage_stats` 仍在 tauri，属于 B 类，需在 P1 中先下沉模块。

### P1：B 类模块下沉（38 个）

**这是覆盖率能否突破 85% 的关键。** 需要先把以下模块/能力下沉到 core：

1. **`services/usage_stats` 模块**（释放 11 个命令：`get_usage_summary`、`get_usage_summary_by_app`、`get_usage_trends`、`get_provider_stats`、`get_model_stats`、`get_request_logs`、`get_request_detail`、`get_model_pricing`、`update_model_pricing`、`delete_model_pricing`、`check_provider_limits`）
2. **`session_manager` 模块**（释放 6 个命令：`delete_session`、`delete_sessions`、`get_session_messages`、`list_sessions`、`sync_session_usage`、`get_usage_data_sources`）
3. **`codex_history_migration` 模块**（释放 2 个命令：`has_codex_unify_history_backup`、`restore_codex_unified_history`）
4. **OAuth AuthState 管理**（释放 8+2 个命令：copilot 认证 + stream_check Copilot 解析）
5. **`SkillService` 网络方法**（释放 7 个命令：发现/安装/更新）
6. **`auto-launch` 能力**（释放 2 个命令：`get_auto_launch_status`、`set_auto_launch`）

### P2：C 类事件拆分（2 个）

- `reset_circuit_breaker`：core 返回结果，由 tauri 发射事件。
- `queryProviderUsage`：core 返回用量快照，由 tauri 发射 `usage-cache-updated` 并刷新托盘。

### P3：D 类命令 Web 兜底（19 个）

这些命令在 Web 端返回明确错误（如 `PlatformError::NotSupported`），前端在 Web 模式下隐藏对应 UI。

### P4：E 类命令评估后迁移（5 个）

- `get_tool_versions`、`probe_tool_installations`、`run_tool_lifecycle_action`：Web 上是否有 Node/npm/这些 CLI？建议增加"工具管理是否启用"配置。
- `check_env_conflicts`、`delete_env_vars`、`restore_env_backup`：Web 上修改环境变量是否有意义？建议默认不暴露或只读检查。

---

## 6. 对附录 D 的修正说明

原附录 D 的部分分类存在以下偏差：

1. **过度归类为"GUI/系统"**：Settings 中 10 个命令实际是 DB 配置读写；Config 中 7 个命令实际是纯业务逻辑。
2. **过度归类为"高级/OAuth/证书"**：Proxy 高级命令 20/21 可以直接迁移。
3. **"依赖未迁移 usage_stats"的再确认**：用量统计命令依赖的 `services/usage_stats` 仍在 `src-tauri`，需要整体下沉到 core 后才能释放这些命令。`session_manager` 和 `session_usage` 服务同样未下沉。
4. **过度归类为"应用专属"**：Hermes/OpenClaw/Workspace 命令绝大多数只是文件读写，完全可迁移。

本分类文档应作为附录 D 的补充和修正。建议在 `migration-roadmap.md` 中把附录 D 的"暂时无法覆盖"口径改为"按阻塞原因分类的待迁移命令"，并优先推进 A 类和 B 类。

---

## 7. P0 批次迁移记录

### 批次 1：proxy.rs 高级配置（20 个 A 命令）

**完成时间**：2026-07-20

**已迁移命令**：

`stop_proxy_with_restore`、`get_proxy_takeover_status`、`set_proxy_takeover_for_app`、`get_proxy_config`、`update_proxy_config`、`get_global_proxy_config`、`update_global_proxy_config`、`get_proxy_config_for_app`、`update_proxy_config_for_app`、`get_default_cost_multiplier`、`set_default_cost_multiplier`、`get_pricing_model_source`、`set_pricing_model_source`、`is_proxy_running`、`is_live_takeover_active`、`switch_proxy_provider`、`get_provider_health`、`get_circuit_breaker_config`、`update_circuit_breaker_config`、`get_circuit_breaker_stats`。

**保留在 tauri 层（C 类）**：

`reset_circuit_breaker`（需发射 `provider-switched` 事件）。

**涉及文件**：

- `src-core/src/commands/proxy.rs`：新增 20 个命令实现。
- `src-core/src/proxy/mod.rs`：re-export `AppProxyConfig`、`GlobalProxyConfig`。
- `src-tauri/src/commands/proxy.rs`：改为调用 core 命令，保留 `reset_circuit_breaker` 的 UI 副作用。
- `src-web/src/routes.rs`：注册 20 个新命令 handler。

**验证结果**：

- `cargo check -p cc-switch-core` ✅ 通过
- `cargo check -p cc-switch-web` ✅ 通过
- `cargo build --bin cc-switch-web` ✅ 通过
- curl 验证 ✅：
  - `get_proxy_config`、`get_global_proxy_config`、`get_proxy_config_for_app` 返回正确结构
  - `get_default_cost_multiplier` 返回 `"1"`
  - `get_circuit_breaker_config` 返回正确熔断器配置
  - `is_proxy_running` 返回 `false`
  - `get_provider_health` 返回 `claude-official` 健康状态
  - `get_circuit_breaker_stats` 返回 `null`（当前 stub 行为）
  - `get_proxy_takeover_status`、`is_live_takeover_active` 返回正确状态

**覆盖率变化**：Web 已暴露命令从 89 个增至 **109 个**，覆盖率从 33.6% 提升至 **~41.1%**。

### 批次 2：`s3_sync.rs` / `webdav_sync.rs`（10 个 A 命令）

**完成时间**：2026-07-20

**已迁移命令**：

- S3：`s3_test_connection`、`s3_sync_upload`、`s3_sync_download`、`s3_sync_save_settings`、`s3_sync_fetch_remote_info`。
- WebDAV：`webdav_test_connection`、`webdav_sync_upload`、`webdav_sync_download`、`webdav_sync_save_settings`、`webdav_sync_fetch_remote_info`。

**涉及文件**：

- `src-core/src/commands/s3_sync.rs`：新增 5 个 S3 同步命令实现（含原 tauri 层的 helper 函数与测试）。
- `src-core/src/commands/webdav_sync.rs`：新增 5 个 WebDAV 同步命令实现（含原 tauri 层的 helper 函数与测试）。
- `src-core/src/commands/mod.rs`：注册 `s3_sync`、`webdav_sync` 模块。
- `src-core/src/services/s3_auto_sync.rs`、`webdav_auto_sync.rs`：将 `AutoSyncSuppressionGuard` 从 `pub(crate)` 改为 `pub`，供 core commands 使用。
- `src-tauri/src/commands/s3_sync.rs`、`webdav_sync.rs`：改为调用 core 命令的薄壳。
- `src-web/src/routes.rs`：注册 10 个新命令 handler。

**验证结果**：

- `cargo check -p cc-switch-core` ✅ 通过
- `cargo check -p cc-switch-web` ✅ 通过
- `cargo build --bin cc-switch-web` ✅ 通过（使用系统 linker）
- curl 验证 ✅：
  - `s3_test_connection`：正确返回连接失败错误
  - `s3_sync_fetch_remote_info`：正确返回"未配置 S3 同步"
  - `s3_sync_save_settings`：保存成功
  - `s3_sync_upload`：正确返回网络错误（无真实 S3）
  - `webdav_test_connection`、`webdav_sync_fetch_remote_info`、`webdav_sync_save_settings`、`webdav_sync_upload` 行为与 S3 侧一致

**覆盖率变化**：Web 已暴露命令从 109 个增至 **119 个**，覆盖率从 ~41.1% 提升至 **~44.9%**。

---

## 8. 验证记录

- `cargo check -p cc-switch-core` ✅ 通过（5 个现有 warning）
- `cargo check -p cc-switch-web` ✅ 通过
- `cargo build --bin cc-switch-web` ✅ 通过
- P0 批次 1 curl 验证 ✅ 通过
