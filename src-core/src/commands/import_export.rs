use crate::database::backup::BackupEntry;
use crate::database::Database;
use crate::error::AppError;
use crate::store::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

/// 导出数据库为 SQL 备份到指定路径。
pub fn export_config_to_file(db: &Database, file_path: &str) -> Result<Value, AppError> {
    let target_path = PathBuf::from(file_path);
    db.export_sql(&target_path)?;
    Ok(json!({
        "success": true,
        "message": "SQL exported successfully",
        "filePath": file_path
    }))
}

/// 从 SQL 备份导入数据库。
///
/// 导入完成后会执行后置同步（当前供应商 → live 配置 + 重新加载设置）。
/// 若同步失败，结果 payload 中仍会包含 `warning` 字段，不会整体报错。
pub fn import_config_from_file(db: &Arc<Database>, file_path: &str) -> Result<Value, AppError> {
    let path_buf = PathBuf::from(file_path);
    let backup_id = db.import_sql(&path_buf)?;
    let db_for_sync = db.clone();
    let warning =
        crate::commands::sync_support::post_sync_warning_from_result(Ok(
            crate::commands::sync_support::run_post_import_sync(db_for_sync),
        ));
    if let Some(msg) = warning.as_ref() {
        log::warn!("[Import] post-import sync warning: {msg}");
    }
    Ok(crate::commands::sync_support::success_payload_with_warning(
        backup_id, warning,
    ))
}

/// 将当前选中的供应商同步到各应用 live 配置。
pub fn sync_current_providers_live(state: &AppState) -> Result<Value, AppError> {
    crate::services::provider::ProviderService::sync_current_to_live(state)?;
    Ok(json!({
        "success": true,
        "message": "Live configuration synchronized"
    }))
}

/// 手动创建数据库备份，返回备份文件名。
pub fn create_db_backup(db: &Database) -> Result<String, AppError> {
    match db.backup_database_file()? {
        Some(path) => Ok(path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default()),
        None => Err(AppError::Config(
            "Database file not found, backup skipped".to_string(),
        )),
    }
}

/// 列出所有数据库备份文件。
pub fn list_db_backups() -> Result<Vec<BackupEntry>, AppError> {
    Database::list_backups()
}

/// 从指定备份文件恢复数据库。
pub fn restore_db_backup(db: &Database, filename: &str) -> Result<String, AppError> {
    db.restore_from_backup(filename)
}

/// 重命名数据库备份文件。
pub fn rename_db_backup(old_filename: &str, new_name: &str) -> Result<String, AppError> {
    Database::rename_backup(old_filename, new_name)
}

/// 删除数据库备份文件。
pub fn delete_db_backup(filename: &str) -> Result<(), AppError> {
    Database::delete_backup(filename)
}
