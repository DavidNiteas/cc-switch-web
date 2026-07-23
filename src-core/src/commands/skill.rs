//! Skills 命令层（无头版）。
//!
//! 仅包含不依赖桌面版网络状态/异步发现服务的同步命令；安装/更新/发现等
//! 保留在桌面版 Tauri 层。

use crate::app_config::{AppType, InstalledSkill, UnmanagedSkill};
use crate::database::Database;
use crate::error::AppError;
use crate::services::skill::{
    ImportSkillSelection, MigrationResult, SkillBackupEntry, SkillRepo, SkillService,
    SkillStorageLocation, SkillUninstallResult,
};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

fn to_app_error<E: std::fmt::Display>(e: E) -> AppError {
    AppError::Message(e.to_string())
}

fn parse_app_type(app: &str) -> Result<AppType, AppError> {
    AppType::from_str(app).map_err(|_| AppError::Message(format!("无效的应用类型: {app}")))
}

/// 获取所有已安装的 Skills。
pub fn get_installed_skills(db: &Arc<Database>) -> Result<Vec<InstalledSkill>, AppError> {
    SkillService::get_all_installed(db).map_err(to_app_error)
}

/// 获取技能备份列表。
pub fn get_skill_backups() -> Result<Vec<SkillBackupEntry>, AppError> {
    SkillService::list_backups().map_err(to_app_error)
}

/// 删除指定技能备份。
pub fn delete_skill_backup(backup_id: &str) -> Result<(), AppError> {
    SkillService::delete_backup(backup_id).map_err(to_app_error)
}

/// 统一卸载 Skill。
pub fn uninstall_skill_unified(
    db: &Arc<Database>,
    id: &str,
) -> Result<SkillUninstallResult, AppError> {
    SkillService::uninstall(db, id).map_err(to_app_error)
}

/// 切换 Skill 的应用启用状态。
pub fn toggle_skill_app(
    db: &Arc<Database>,
    id: &str,
    app: &str,
    enabled: bool,
) -> Result<(), AppError> {
    let app_type = parse_app_type(app)?;
    SkillService::toggle_app(db, id, &app_type, enabled).map_err(to_app_error)
}

/// 扫描未管理的 Skills。
pub fn scan_unmanaged_skills(db: &Arc<Database>) -> Result<Vec<UnmanagedSkill>, AppError> {
    SkillService::scan_unmanaged(db).map_err(to_app_error)
}

/// 从应用目录导入 Skills。
pub fn import_skills_from_apps(
    db: &Arc<Database>,
    imports: Vec<ImportSkillSelection>,
) -> Result<Vec<InstalledSkill>, AppError> {
    SkillService::import_from_apps(db, imports).map_err(to_app_error)
}

/// 获取技能仓库列表。
pub fn get_skill_repos(db: &Arc<Database>) -> Result<Vec<SkillRepo>, AppError> {
    db.get_skill_repos()
}

/// 添加/更新技能仓库。
pub fn add_skill_repo(db: &Arc<Database>, repo: SkillRepo) -> Result<(), AppError> {
    db.save_skill_repo(&repo)
}

/// 删除技能仓库。
pub fn remove_skill_repo(db: &Arc<Database>, owner: &str, name: &str) -> Result<(), AppError> {
    db.delete_skill_repo(owner, name)
}

/// 从 ZIP 文件安装 Skills。
pub fn install_skills_from_zip(
    db: &Arc<Database>,
    file_path: &str,
    current_app: &str,
) -> Result<Vec<InstalledSkill>, AppError> {
    let app_type = parse_app_type(current_app)?;
    let path = Path::new(file_path);
    SkillService::install_from_zip(db, path, &app_type).map_err(to_app_error)
}

/// 迁移 Skill 存储位置。
pub fn migrate_skill_storage(
    db: &Arc<Database>,
    target: SkillStorageLocation,
) -> Result<MigrationResult, AppError> {
    SkillService::migrate_storage(db, target).map_err(to_app_error)
}

/// 从备份恢复 Skill（兼容旧 API，等价于新版的 restore_from_backup）。
pub fn restore_skill_backup(
    db: &Arc<Database>,
    backup_id: &str,
    current_app: &str,
) -> Result<InstalledSkill, AppError> {
    let app_type = parse_app_type(current_app)?;
    SkillService::restore_from_backup(db, backup_id, &app_type).map_err(to_app_error)
}

/// 卸载 Skill（旧 API，等价于 `uninstall_skill_for_app("claude", directory)`）。
pub fn uninstall_skill(
    db: &Arc<Database>,
    directory: &str,
) -> Result<SkillUninstallResult, AppError> {
    uninstall_skill_for_app(db, "claude", directory)
}

/// 卸载指定应用的 Skill（兼容旧 API）。
///
/// 旧 API 通过 directory 定位 skill：先在已安装列表中按目录大小写不敏感匹配，
/// 找到后调用 `SkillService::uninstall(db, id)` 完成卸载。
pub fn uninstall_skill_for_app(
    db: &Arc<Database>,
    app: &str,
    directory: &str,
) -> Result<SkillUninstallResult, AppError> {
    let _ = parse_app_type(app)?; // 验证 app 参数有效

    let skills = get_installed_skills(db)?;
    let skill = skills
        .into_iter()
        .find(|s| s.directory.eq_ignore_ascii_case(directory))
        .ok_or_else(|| AppError::Message(format!("未找到已安装的 Skill: {directory}")))?;

    SkillService::uninstall(db, &skill.id).map_err(to_app_error)
}

// ============================================================================
// SkillService 网络方法（B 类，已下沉到 core）
// ============================================================================

use crate::services::skill::{DiscoverableSkill, Skill, SkillUpdateInfo, SkillsShSearchResult};

/// 获取全局 SkillService 单例。
///
/// SkillService 是无状态的（网络/文件操作都通过参数传入），可以安全地
/// 作为单例共享。Web 模式下也使用同一个实例。
fn skill_service() -> SkillService {
    SkillService::new()
}

/// 列出所有可发现 + 已安装的技能（合并视图，兼容旧 API）。
pub async fn get_skills(db: &Arc<Database>) -> Result<Vec<Skill>, AppError> {
    let repos = db.get_skill_repos().map_err(to_app_error)?;
    skill_service()
        .list_skills(repos, db)
        .await
        .map_err(to_app_error)
}

/// `get_skills` 的应用参数化版本（兼容旧 API；新版本不再区分应用）。
pub async fn get_skills_for_app(db: &Arc<Database>, app: &str) -> Result<Vec<Skill>, AppError> {
    let _ = parse_app_type(app)?;
    get_skills(db).await
}

/// 发现所有可用技能（从启用的仓库拉取）。
pub async fn discover_available_skills(
    db: &Arc<Database>,
) -> Result<Vec<DiscoverableSkill>, AppError> {
    let repos = db.get_skill_repos().map_err(to_app_error)?;
    skill_service()
        .discover_available(repos)
        .await
        .map_err(to_app_error)
}

/// 统一安装 Skill（按 current_app 启用对应应用）。
pub async fn install_skill_unified(
    db: &Arc<Database>,
    skill: DiscoverableSkill,
    current_app: &str,
) -> Result<InstalledSkill, AppError> {
    let app_type = parse_app_type(current_app)?;
    skill_service()
        .install(db, &skill, &app_type)
        .await
        .map_err(to_app_error)
}

/// 兼容旧 API：通过 directory 在可发现列表中找到 skill 后安装到 claude。
pub async fn install_skill(db: &Arc<Database>, directory: &str) -> Result<bool, AppError> {
    install_skill_for_app(db, "claude", directory).await
}

/// 兼容旧 API：通过 directory 在可发现列表中找到 skill 后安装到指定 app。
pub async fn install_skill_for_app(
    db: &Arc<Database>,
    app: &str,
    directory: &str,
) -> Result<bool, AppError> {
    let app_type = parse_app_type(app)?;
    let repos = db.get_skill_repos().map_err(to_app_error)?;
    let skills = skill_service()
        .discover_available(repos)
        .await
        .map_err(to_app_error)?;
    let target = skills
        .into_iter()
        .find(|s| s.directory.eq_ignore_ascii_case(directory))
        .ok_or_else(|| AppError::Message(format!("未找到可安装的 Skill: {directory}")))?;
    skill_service()
        .install(db, &target, &app_type)
        .await
        .map_err(to_app_error)?;
    Ok(true)
}

/// 检查所有已安装 skill 的更新。
pub async fn check_skill_updates(db: &Arc<Database>) -> Result<Vec<SkillUpdateInfo>, AppError> {
    skill_service()
        .check_updates(db)
        .await
        .map_err(to_app_error)
}

/// 更新单个 skill 到最新版本。
pub async fn update_skill(db: &Arc<Database>, id: &str) -> Result<InstalledSkill, AppError> {
    skill_service()
        .update_skill(db, id)
        .await
        .map_err(to_app_error)
}

/// 在 skills.sh 公共目录搜索技能。
pub async fn search_skills_sh(
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<SkillsShSearchResult, AppError> {
    SkillService::search_skills_sh(query, limit, offset)
        .await
        .map_err(to_app_error)
}
