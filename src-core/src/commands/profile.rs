use crate::database::Profile;
use crate::error::AppError;
use crate::services::profile::{ProfilePayload, ProfileScope, ProfileService};
use crate::store::AppState;
use serde::Serialize;

/// Profile 传输对象
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDto {
    pub id: String,
    pub name: String,
    pub payload: ProfilePayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

impl From<Profile> for ProfileDto {
    fn from(profile: Profile) -> Self {
        let payload = serde_json::from_str(&profile.payload).unwrap_or_else(|e| {
            log::warn!(
                "解析 profile '{}' payload 失败，使用默认值: {e}",
                profile.id
            );
            ProfilePayload::default()
        });
        Self {
            id: profile.id,
            name: profile.name,
            payload,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        }
    }
}

/// 每个分组当前激活的项目 id
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentProfileIds {
    pub claude: Option<String>,
    pub claude_desktop: Option<String>,
    pub codex: Option<String>,
}

/// Profile 列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesResponse {
    pub profiles: Vec<ProfileDto>,
    pub current_ids: CurrentProfileIds,
}

/// 获取所有项目与当前激活 id。
pub fn get_profiles(state: &AppState) -> Result<ProfilesResponse, AppError> {
    let profiles = ProfileService::list(state)?;
    let current_ids = CurrentProfileIds {
        claude: state
            .db
            .get_current_profile_id(ProfileScope::Claude.as_str())?,
        claude_desktop: state
            .db
            .get_current_profile_id(ProfileScope::ClaudeDesktop.as_str())?,
        codex: state
            .db
            .get_current_profile_id(ProfileScope::Codex.as_str())?,
    };
    Ok(ProfilesResponse {
        profiles: profiles.into_iter().map(ProfileDto::from).collect(),
        current_ids,
    })
}

/// 创建项目。
pub fn create_profile(state: &AppState, name: &str, scope: &str) -> Result<ProfileDto, AppError> {
    let scope = ProfileScope::parse(scope)?;
    ProfileService::create(state, name, scope).map(ProfileDto::from)
}

/// 更新项目。
pub fn update_profile(
    state: &AppState,
    id: &str,
    name: Option<String>,
    resnapshot: Option<bool>,
    scope: Option<&str>,
) -> Result<ProfileDto, AppError> {
    let scope = scope.map(ProfileScope::parse).transpose()?;
    ProfileService::update(state, id, name, resnapshot.unwrap_or(false), scope)
        .map(ProfileDto::from)
}

/// 删除项目。
pub fn delete_profile(state: &AppState, id: &str) -> Result<(), AppError> {
    ProfileService::delete(state, id)
}

/// 应用项目快照。
/// 返回 (warnings, should_stop_proxy)。调用方（tauri/web 壳）应根据需要停止代理并发送事件。
pub fn apply_profile(
    state: &AppState,
    id: &str,
    scope: &str,
) -> Result<(Vec<String>, bool), AppError> {
    let scope = ProfileScope::parse(scope)?;
    ProfileService::apply(state, id, scope)
}

/// 清除当前分组的项目绑定。
pub fn clear_current_profile(state: &AppState, scope: &str) -> Result<(), AppError> {
    let scope = ProfileScope::parse(scope)?;
    state.db.set_current_profile_id(scope.as_str(), None)
}
