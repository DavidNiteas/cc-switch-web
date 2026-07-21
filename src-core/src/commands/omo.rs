use crate::database::Database;
use crate::error::AppError;
use crate::services::omo::{OmoLocalFileData, OmoVariant, SLIM, STANDARD};
use crate::services::OmoService;

/// 读取 OMO 本地文件。
pub fn read_omo_local_file() -> Result<OmoLocalFileData, AppError> {
    OmoService::read_local_file(&STANDARD)
}

/// 获取当前 OMO 供应商 ID。
pub fn get_current_omo_provider_id(db: &Database) -> Result<String, AppError> {
    let provider = db.get_current_omo_provider("opencode", "omo")?;
    Ok(provider.map(|p| p.id).unwrap_or_default())
}

/// 禁用当前 OMO 供应商。
pub fn disable_current_omo(db: &Database) -> Result<(), AppError> {
    let providers = db.get_all_providers("opencode")?;
    for (id, p) in &providers {
        if p.category.as_deref() == Some("omo") {
            db.clear_omo_provider_current("opencode", id, "omo")?;
        }
    }
    OmoService::delete_config_file(&STANDARD)
}

/// 读取 OMO Slim 本地文件。
pub fn read_omo_slim_local_file() -> Result<OmoLocalFileData, AppError> {
    OmoService::read_local_file(&SLIM)
}

/// 获取当前 OMO Slim 供应商 ID。
pub fn get_current_omo_slim_provider_id(db: &Database) -> Result<String, AppError> {
    let provider = db.get_current_omo_provider("opencode", "omo-slim")?;
    Ok(provider.map(|p| p.id).unwrap_or_default())
}

/// 禁用当前 OMO Slim 供应商。
pub fn disable_current_omo_slim(db: &Database) -> Result<(), AppError> {
    let providers = db.get_all_providers("opencode")?;
    for (id, p) in &providers {
        if p.category.as_deref() == Some("omo-slim") {
            db.clear_omo_provider_current("opencode", id, "omo-slim")?;
        }
    }
    OmoService::delete_config_file(&SLIM)
}
