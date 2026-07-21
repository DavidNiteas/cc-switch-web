//! Skills 服务层
//!
//! v3.10.0+ 统一管理架构：
//! - SSOT（单一事实源）：`~/.cc-switch/skills/`
//! - 安装时下载到 SSOT，按需同步到各应用目录
//! - 数据库存储安装记录和启用状态
//!
//! 本模块已迁移至 `cc-switch-core`，tauri 侧仅做 re-export 以保持现有 import 路径兼容。

pub use cc_switch_core::services::skill::*;
