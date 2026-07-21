//! Deep-link 导入命令（A 类）。
//!
//! 对应 tauri 侧 `commands/deeplink.rs` 的 4 个 A 类命令。
//! 实现已下沉到 `cc_switch_core::deeplink`。

use crate::deeplink::{
    import_mcp_from_deeplink, import_prompt_from_deeplink, import_provider_from_deeplink,
    import_skill_from_deeplink, parse_and_merge_config, parse_deeplink_url, DeepLinkImportRequest,
};
use crate::error::AppError;
use crate::store::AppState;
use serde_json::{json, Value};

/// 解析 deep link URL，返回待确认的导入请求（前端确认对话框）。
pub fn parse_deeplink(url: &str) -> Result<DeepLinkImportRequest, AppError> {
    log::info!("Parsing deep link URL: {url}");
    parse_deeplink_url(url)
}

/// 合并 Base64/URL 中的配置到 deep link 请求。
pub fn merge_deeplink_config(
    request: DeepLinkImportRequest,
) -> Result<DeepLinkImportRequest, AppError> {
    log::info!("Merging config for deep link request: {:?}", request.name);
    parse_and_merge_config(&request)
}

/// 旧版 provider 导入（保留兼容性）。
pub fn import_from_deeplink(
    state: &AppState,
    request: DeepLinkImportRequest,
) -> Result<String, AppError> {
    log::info!(
        "Importing provider from deep link: {:?} for app {:?}",
        request.name,
        request.app
    );
    let provider_id = import_provider_from_deeplink(state, request)?;
    log::info!("Successfully imported provider with ID: {provider_id}");
    Ok(provider_id)
}

/// 统一 deep-link 资源导入：根据 `request.resource` 分发到 provider/prompt/mcp/skill。
pub async fn import_from_deeplink_unified(
    state: &AppState,
    request: DeepLinkImportRequest,
) -> Result<Value, AppError> {
    log::info!("Importing {} resource from deep link", request.resource);
    match request.resource.as_str() {
        "provider" => {
            let provider_id = import_provider_from_deeplink(state, request)?;
            Ok(json!({
                "type": "provider",
                "id": provider_id
            }))
        }
        "prompt" => {
            let prompt_id = import_prompt_from_deeplink(state, request)?;
            Ok(json!({
                "type": "prompt",
                "id": prompt_id
            }))
        }
        "mcp" => {
            let result = import_mcp_from_deeplink(state, request)?;
            Ok(json!({
                "type": "mcp",
                "importedCount": result.imported_count,
                "importedIds": result.imported_ids,
                "failed": result.failed
            }))
        }
        "skill" => {
            let skill_key = import_skill_from_deeplink(state, request)?;
            Ok(json!({
                "type": "skill",
                "key": skill_key
            }))
        }
        other => Err(AppError::Message(format!(
            "Unsupported resource type: {other}"
        ))),
    }
}
