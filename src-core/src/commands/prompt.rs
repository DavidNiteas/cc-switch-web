use crate::app_config::AppType;
use crate::error::AppError;
use crate::prompt::Prompt;
use crate::services::PromptService;
use crate::store::AppState;
use indexmap::IndexMap;
use std::str::FromStr;

/// 获取指定应用的所有提示词。
pub fn get_prompts(
    state: &AppState,
    app: &str,
) -> Result<IndexMap<String, Prompt>, AppError> {
    let app_type = AppType::from_str(app)?;
    PromptService::get_prompts(state, app_type)
}

/// 新增或更新提示词。
pub fn upsert_prompt(
    state: &AppState,
    app: &str,
    id: &str,
    prompt: Prompt,
) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    PromptService::upsert_prompt(state, app_type, id, prompt)
}

/// 删除提示词。
pub fn delete_prompt(state: &AppState, app: &str, id: &str) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    PromptService::delete_prompt(state, app_type, id)
}

/// 启用指定提示词（互斥激活）。
pub fn enable_prompt(state: &AppState, app: &str, id: &str) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    PromptService::enable_prompt(state, app_type, id)
}

/// 从现有提示词文件导入。
pub fn import_prompt_from_file(state: &AppState, app: &str) -> Result<String, AppError> {
    let app_type = AppType::from_str(app)?;
    PromptService::import_from_file(state, app_type)
}

/// 获取当前 live 提示词文件内容。
pub fn get_current_prompt_file_content(app: &str) -> Result<Option<String>, AppError> {
    let app_type = AppType::from_str(app)?;
    PromptService::get_current_file_content(app_type)
}
