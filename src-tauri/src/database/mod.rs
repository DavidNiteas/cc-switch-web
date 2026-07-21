//! 数据库模块
//!
//! 已迁移到 `cc-switch-core`，本模块仅做 re-export 以保持现有 import 路径兼容。

pub use cc_switch_core::database::*;
// `#[macro_export]` 宏默认位于 crate root，显式 re-export 到本模块以便 `crate::database::lock_conn!` 继续工作。
pub use cc_switch_core::lock_conn;
