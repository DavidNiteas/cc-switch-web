//! 环境变量管理命令（E 类，已下沉到 core）。
//!
//! 对应 tauri 侧 `commands/env.rs` 的 3 个命令。底层服务 `env_checker.rs` 与
//! `env_manager.rs` 已迁移到 core。Web 模式下用户应该明确知道：修改的是
//! **运行 cc-switch-web 服务的服务器**上的环境变量文件，不是本地桌面。

use crate::error::AppError;
use crate::services::env_checker::{check_env_conflicts as check_conflicts, EnvConflict};
use crate::services::env_manager::{
    delete_env_vars as delete_vars, restore_from_backup, BackupInfo,
};

/// 检查指定应用的本地环境变量冲突。
pub fn check_env_conflicts(app: &str) -> Result<Vec<EnvConflict>, AppError> {
    check_conflicts(app).map_err(|e| AppError::Message(e))
}

/// 删除环境变量（带自动备份）。
///
/// **Web 模式风险提示**：此命令修改服务器上当前用户的 shell rc 文件
/// （如 ~/.zshrc、~/.bashrc），不是访问 Web UI 的用户本地文件。
/// 前端 UI 应明确告知用户这一点。
pub fn delete_env_vars(conflicts: Vec<EnvConflict>) -> Result<BackupInfo, AppError> {
    delete_vars(conflicts).map_err(|e| AppError::Message(e))
}

/// 从备份文件恢复环境变量。
pub fn restore_env_backup(backup_path: &str) -> Result<(), AppError> {
    restore_from_backup(backup_path.to_string()).map_err(|e| AppError::Message(e))
}
