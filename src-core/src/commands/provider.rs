use crate::app_config::AppType;
use crate::database::Database;
use crate::error::AppError;
use crate::provider::Provider;
use indexmap::IndexMap;
use std::str::FromStr;
use std::sync::Arc;

/// 获取指定应用的所有供应商。
pub fn get_providers(
    db: &Arc<Database>,
    app: &str,
) -> Result<IndexMap<String, Provider>, AppError> {
    let app_type = AppType::from_str(app)?;
    db.get_all_providers(app_type.as_str())
}

/// 获取当前激活的供应商 ID。
pub fn get_current_provider_id(db: &Arc<Database>, app: &str) -> Result<Option<String>, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::settings::get_effective_current_provider(db, &app_type)
}

/// 检查 providers 表是否为空。
pub fn is_providers_empty(db: &Arc<Database>) -> Result<bool, AppError> {
    db.is_providers_empty()
}

/// 初始化默认官方供应商（启动时 seed）。
pub fn init_default_official_providers(db: &Arc<Database>) -> Result<usize, AppError> {
    db.init_default_official_providers()
}

/// 新增或更新供应商。
pub fn save_provider(
    state: &crate::store::AppState,
    app: &str,
    provider: Provider,
    original_id: Option<&str>,
) -> Result<bool, AppError> {
    let app_type = AppType::from_str(app)?;
    if let Some(original_id) = original_id {
        crate::services::ProviderService::update(state, app_type, Some(original_id), provider)
    } else {
        crate::services::ProviderService::add(state, app_type, provider, true)
    }
}

/// 删除供应商。
pub fn delete_provider(
    state: &crate::store::AppState,
    app: &str,
    id: &str,
) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::delete(state, app_type, id)
}

/// 切换当前供应商。
pub fn switch_provider(
    state: &crate::store::AppState,
    app: &str,
    id: &str,
) -> Result<crate::services::SwitchResult, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::switch(state, app_type, id)
}

/// 获取指定供应商的自定义端点列表。
pub fn get_custom_endpoints(
    state: &crate::store::AppState,
    app: &str,
    provider_id: &str,
) -> Result<Vec<crate::settings::CustomEndpoint>, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::get_custom_endpoints(state, app_type, provider_id)
}

/// 添加自定义端点。
pub fn add_custom_endpoint(
    state: &crate::store::AppState,
    app: &str,
    provider_id: &str,
    url: &str,
) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::add_custom_endpoint(state, app_type, provider_id, url.to_string())
}

/// 移除自定义端点。
pub fn remove_custom_endpoint(
    state: &crate::store::AppState,
    app: &str,
    provider_id: &str,
    url: &str,
) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::remove_custom_endpoint(state, app_type, provider_id, url.to_string())
}

/// 更新端点最后使用时间。
pub fn update_endpoint_last_used(
    state: &crate::store::AppState,
    app: &str,
    provider_id: &str,
    url: &str,
) -> Result<(), AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::update_endpoint_last_used(state, app_type, provider_id, url.to_string())
}

/// 更新供应商排序。
pub fn update_providers_sort_order(
    state: &crate::store::AppState,
    app: &str,
    updates: Vec<crate::services::ProviderSortUpdate>,
) -> Result<bool, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::update_sort_order(state, app_type, updates)
}

/// 导入默认配置。
pub fn import_default_config(
    state: &crate::store::AppState,
    app: &str,
) -> Result<bool, AppError> {
    let app_type = AppType::from_str(app)?;
    let imported = crate::services::ProviderService::import_default_config(state, app_type.clone())?;

    if imported {
        if state
            .db
            .should_auto_extract_config_snippet(app_type.as_str())?
        {
            match crate::services::ProviderService::extract_common_config_snippet(state, app_type.clone()) {
                Ok(snippet) if !snippet.is_empty() && snippet != "{}" => {
                    let _ = state
                        .db
                        .set_config_snippet(app_type.as_str(), Some(snippet));
                    let _ = state
                        .db
                        .set_config_snippet_cleared(app_type.as_str(), false);
                }
                _ => {}
            }
        }

        crate::services::ProviderService::migrate_legacy_common_config_usage_if_needed(state, app_type)?;
    }

    Ok(imported)
}

/// 获取所有通用供应商。
pub fn get_universal_providers(
    state: &crate::store::AppState,
) -> Result<std::collections::HashMap<String, crate::provider::UniversalProvider>, AppError> {
    crate::services::ProviderService::list_universal(state)
}

/// 获取指定通用供应商。
pub fn get_universal_provider(
    state: &crate::store::AppState,
    id: &str,
) -> Result<Option<crate::provider::UniversalProvider>, AppError> {
    crate::services::ProviderService::get_universal(state, id)
}

/// 新增或更新通用供应商。
pub fn upsert_universal_provider(
    state: &crate::store::AppState,
    provider: crate::provider::UniversalProvider,
) -> Result<bool, AppError> {
    crate::services::ProviderService::upsert_universal(state, provider)
}

/// 删除通用供应商。
pub fn delete_universal_provider(
    state: &crate::store::AppState,
    id: &str,
) -> Result<bool, AppError> {
    crate::services::ProviderService::delete_universal(state, id)
}

/// 同步通用供应商到各应用。
pub fn sync_universal_provider(
    state: &crate::store::AppState,
    id: &str,
) -> Result<bool, AppError> {
    crate::services::ProviderService::sync_universal_to_apps(state, id)
}

/// 从 OpenCode live 配置导入供应商。
pub fn import_opencode_providers_from_live(
    state: &crate::store::AppState,
) -> Result<usize, AppError> {
    crate::services::provider::import_opencode_providers_from_live(state)
}

/// 获取 OpenCode live 中的供应商 id 列表。
pub fn get_opencode_live_provider_ids() -> Result<Vec<String>, AppError> {
    crate::opencode_config::get_providers()
        .map(|providers| providers.keys().cloned().collect())
}

/// 新增供应商（add 路径，区别于 save_provider 的 upsert 语义）。
pub fn add_provider(
    state: &crate::store::AppState,
    app: &str,
    provider: Provider,
    add_to_live: Option<bool>,
) -> Result<bool, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::add(state, app_type, provider, add_to_live.unwrap_or(true))
}

/// 更新供应商（原 id 可选，用于重命名场景）。
pub fn update_provider(
    state: &crate::store::AppState,
    app: &str,
    provider: Provider,
    original_id: Option<&str>,
) -> Result<bool, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::update(state, app_type, original_id, provider)
}

/// 从 live 配置中移除指定供应商。
pub fn remove_provider_from_live_config(
    state: &crate::store::AppState,
    app: &str,
    id: &str,
) -> Result<bool, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::remove_from_live_config(state, app_type, id).map(|_| true)
}

/// 读取指定应用的 live provider 配置（json 形式）。
pub fn read_live_provider_settings(app: &str) -> Result<serde_json::Value, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::read_live_settings(app_type)
}

/// 在线测试 usage script 的执行结果。
pub async fn test_usage_script(
    state: &crate::store::AppState,
    provider_id: &str,
    app: &str,
    script_code: &str,
    timeout: Option<u64>,
    api_key: Option<&str>,
    base_url: Option<&str>,
    access_token: Option<&str>,
    user_id: Option<&str>,
    template_type: Option<&str>,
) -> Result<crate::provider::UsageResult, AppError> {
    let app_type = AppType::from_str(app)?;
    crate::services::ProviderService::test_usage_script(
        state,
        app_type,
        provider_id,
        script_code,
        timeout.unwrap_or(10),
        api_key,
        base_url,
        access_token,
        user_id,
        template_type,
    )
    .await
}

/// 确保 Claude Desktop 官方供应商 seed 存在。
pub fn ensure_claude_desktop_official_provider(
    db: &Arc<Database>,
) -> Result<bool, AppError> {
    db.ensure_official_seed_by_id(
        crate::database::CLAUDE_DESKTOP_OFFICIAL_PROVIDER_ID,
        AppType::ClaudeDesktop,
    )
}

/// 确保 Codex 官方供应商 seed 存在。
pub fn ensure_codex_official_provider(db: &Arc<Database>) -> Result<bool, AppError> {
    db.ensure_official_seed_by_id(crate::database::CODEX_OFFICIAL_PROVIDER_ID, AppType::Codex)
}

/// 获取 Claude Desktop 默认代理路由表。
pub fn get_claude_desktop_default_routes()
-> Vec<crate::claude_desktop_config::ClaudeDesktopDefaultRoute> {
    crate::claude_desktop_config::default_proxy_routes()
}

/// 获取 Claude Desktop 配置状态（运行中/proxy 状态/路由等）。
pub async fn get_claude_desktop_status(
    state: &crate::store::AppState,
) -> Result<crate::claude_desktop_config::ClaudeDesktopStatus, AppError> {
    let proxy_running = state.proxy_service.is_running().await;
    crate::claude_desktop_config::get_status(state.db.as_ref(), proxy_running)
}

/// 从 Claude providers 导入到 ClaudeDesktop providers（含路由派生）。
pub fn import_claude_desktop_providers_from_claude(
    state: &crate::store::AppState,
) -> Result<usize, AppError> {
    use crate::provider::ClaudeDesktopMode;
    use std::collections::HashSet;

    let claude_providers = state
        .db
        .get_all_providers(AppType::Claude.as_str())?;
    let existing_ids = state
        .db
        .get_provider_ids(AppType::ClaudeDesktop.as_str())?;
    let existing_set: HashSet<_> = existing_ids.iter().cloned().collect();

    let mut imported = 0usize;
    for provider in claude_providers.values() {
        if existing_set.contains(&provider.id) {
            continue;
        }

        let mut desktop_provider = provider.clone();
        desktop_provider.in_failover_queue = false;
        let meta = desktop_provider
            .meta
            .get_or_insert_with(crate::provider::ProviderMeta::default);

        if crate::claude_desktop_config::is_compatible_direct_provider(provider)
            && crate::claude_desktop_config::claude_provider_models_are_claude_safe(provider)
        {
            meta.claude_desktop_mode = Some(ClaudeDesktopMode::Direct);
        } else if let Some(routes) =
            crate::claude_desktop_config::suggested_claude_desktop_routes(provider)
        {
            meta.claude_desktop_mode = Some(ClaudeDesktopMode::Proxy);
            meta.claude_desktop_model_routes = routes;
        } else {
            continue;
        }

        state
            .db
            .save_provider(AppType::ClaudeDesktop.as_str(), &desktop_provider)?;
        imported += 1;
    }

    // 用户主动 import 是"重新整理 ClaudeDesktop 表"的隐式信号，把官方入口补回来。
    if let Err(e) = state.db.ensure_official_seed_by_id(
        crate::database::CLAUDE_DESKTOP_OFFICIAL_PROVIDER_ID,
        AppType::ClaudeDesktop,
    ) {
        log::warn!("Failed to ensure claude-desktop-official seed during import: {e}");
    }

    Ok(imported)
}

// ============================================================================
// queryProviderUsage — C 类事件拆分
// ============================================================================

const TEMPLATE_TYPE_GITHUB_COPILOT: &str = "github_copilot";
const TEMPLATE_TYPE_TOKEN_PLAN: &str = "token_plan";
const TEMPLATE_TYPE_BALANCE: &str = "balance";
const TEMPLATE_TYPE_OFFICIAL_SUBSCRIPTION: &str = "official_subscription";
const COPILOT_UNIT_PREMIUM: &str = "requests";

/// 解析 `(base_url, api_key)` 用于通用 usage 查询。
fn resolve_native_credentials(app_type: &AppType, provider: Option<&Provider>) -> (String, String) {
    provider
        .map(|p| p.resolve_usage_credentials(app_type))
        .unwrap_or_default()
}

/// 解析 Coding Plan 路径下的 `(base_url, api_key)`。
fn resolve_coding_plan_credentials(
    app_type: &AppType,
    provider: Option<&Provider>,
    usage_script: Option<&crate::provider::UsageScript>,
) -> (String, String) {
    use crate::provider::UsageScript;
    let is_zenmux = usage_script
        .and_then(|s| s.coding_plan_provider.as_deref())
        .map(|p| p == "zenmux")
        .unwrap_or(false);
    if is_zenmux {
        return (
            usage_script.and_then(|s| s.base_url.clone()).unwrap_or_default(),
            usage_script.and_then(|s| s.api_key.clone()).unwrap_or_default(),
        );
    }
    let native = resolve_native_credentials(app_type, provider);
    let base_url = usage_script
        .and_then(|s| s.base_url.as_deref())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or(native.0);
    let api_key = usage_script
        .and_then(|s| s.api_key.as_deref())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or(native.1);
    (base_url, api_key)
}

/// 查询指定 provider 的使用量。
///
/// 内部按 `template_type` 路由到 5 个查询路径：Copilot / Coding Plan /
/// Balance / Subscription / 通用 JS 脚本。返回 `UsageResult`。
///
/// **C 类副作用说明**：调用方（tauri/web 外壳）应在拿到 `Ok(snapshot)` 后
/// 自行发射 `usage-cache-updated` 事件并刷新托盘/UI。core 本身不发射事件，
/// 只在 `state.usage_cache` 内写入缓存。
pub async fn query_provider_usage(
    state: &crate::store::AppState,
    provider_id: &str,
    app: &str,
) -> Result<crate::provider::UsageResult, AppError> {
    use crate::provider::{UsageData, UsageResult};

    let app_type = AppType::from_str(app)?;
    let providers = state.db.get_all_providers(app_type.as_str())?;
    let provider = providers.get(provider_id);
    let usage_script = provider
        .and_then(|p| p.meta.as_ref())
        .and_then(|m| m.usage_script.as_ref());
    let template_type = usage_script
        .and_then(|s| s.template_type.as_deref())
        .unwrap_or("");

    // ── GitHub Copilot 专用路径 ──
    if template_type == TEMPLATE_TYPE_GITHUB_COPILOT {
        let copilot_account_id = provider
            .and_then(|p| p.meta.as_ref())
            .and_then(|m| m.managed_account_id_for(TEMPLATE_TYPE_GITHUB_COPILOT));

        let copilot_arc = state.proxy_service.auth_state().copilot.clone();
        let auth_manager = copilot_arc.read().await;
        let usage = match copilot_account_id.as_deref() {
            Some(account_id) => auth_manager
                .fetch_usage_for_account(account_id)
                .await
                .map_err(|e| AppError::Message(format!("Failed to fetch Copilot usage: {e}")))?,
            None => auth_manager
                .fetch_usage()
                .await
                .map_err(|e| AppError::Message(format!("Failed to fetch Copilot usage: {e}")))?,
        };
        let premium = &usage.quota_snapshots.premium_interactions;
        let used = premium.entitlement - premium.remaining;

        return Ok(UsageResult {
            success: true,
            data: Some(vec![UsageData {
                plan_name: Some(usage.copilot_plan),
                remaining: Some(premium.remaining as f64),
                total: Some(premium.entitlement as f64),
                used: Some(used as f64),
                unit: Some(COPILOT_UNIT_PREMIUM.to_string()),
                is_valid: Some(true),
                invalid_message: None,
                extra: Some(format!("Reset: {}", usage.quota_reset_date)),
            }]),
            error: None,
        });
    }

    // ── Coding Plan 专用路径 ──
    if template_type == TEMPLATE_TYPE_TOKEN_PLAN {
        let (base_url, api_key) =
            resolve_coding_plan_credentials(&app_type, provider, usage_script);
        let access_key_id = usage_script.and_then(|s| s.access_key_id.clone());
        let secret_access_key = usage_script.and_then(|s| s.secret_access_key.clone());
        let coding_plan_provider = usage_script.and_then(|s| s.coding_plan_provider.clone());
        let team_organization_id = usage_script.and_then(|s| s.team_organization_id.clone());
        let team_project_id = usage_script.and_then(|s| s.team_project_id.clone());

        let quota = crate::services::coding_plan::get_coding_plan_quota(
            &base_url,
            &api_key,
            access_key_id.as_deref(),
            secret_access_key.as_deref(),
            coding_plan_provider.as_deref(),
            team_organization_id.as_deref(),
            team_project_id.as_deref(),
        )
        .await
        .map_err(|e| AppError::Message(format!("Failed to query coding plan: {e}")))?;

        if !quota.success {
            return Ok(UsageResult {
                success: false,
                data: None,
                error: quota.error,
            });
        }

        let has_usd = quota
            .tiers
            .first()
            .map(|t| t.used_value_usd.is_some())
            .unwrap_or(false);
        let plan_label = quota
            .credential_message
            .as_deref()
            .and_then(|msg| msg.split(' ').next())
            .map(|tier| format!("ZenMux·{}", tier.to_uppercase()));
        let mut first_tier = true;

        let data: Vec<UsageData> = quota
            .tiers
            .iter()
            .map(|tier| {
                let total = 100.0;
                let used = tier.utilization;
                let remaining = total - used;
                let extra = if has_usd {
                    let mut extra_json = serde_json::json!({
                        "resetsAt": tier.resets_at,
                    });
                    if let Some(v) = tier.used_value_usd {
                        extra_json["usedValueUsd"] = serde_json::json!(v);
                    }
                    if let Some(v) = tier.max_value_usd {
                        extra_json["maxValueUsd"] = serde_json::json!(v);
                    }
                    if first_tier {
                        if let Some(ref label) = plan_label {
                            extra_json["planLabel"] = serde_json::json!(label);
                        }
                        first_tier = false;
                    }
                    Some(extra_json.to_string())
                } else {
                    tier.resets_at.clone()
                };
                UsageData {
                    plan_name: Some(tier.name.clone()),
                    remaining: Some(remaining),
                    total: Some(total),
                    used: Some(used),
                    unit: Some("%".to_string()),
                    is_valid: Some(true),
                    invalid_message: None,
                    extra,
                }
            })
            .collect();

        return Ok(UsageResult {
            success: true,
            data: if data.is_empty() { None } else { Some(data) },
            error: None,
        });
    }

    // ── 官方余额查询路径 ──
    if template_type == TEMPLATE_TYPE_BALANCE {
        let (base_url, api_key) = resolve_native_credentials(&app_type, provider);
        return crate::services::balance::get_balance(&base_url, &api_key)
            .await
            .map_err(|e| AppError::Message(format!("Failed to query balance: {e}")));
    }

    // ── 官方订阅额度查询路径 ──
    if template_type == TEMPLATE_TYPE_OFFICIAL_SUBSCRIPTION {
        if !usage_script.map(|s| s.enabled).unwrap_or(false) {
            return Ok(UsageResult {
                success: false,
                data: None,
                error: Some("Usage query is disabled".to_string()),
            });
        }

        let quota = crate::services::subscription::get_subscription_quota(app_type.as_str())
            .await
            .map_err(|e| AppError::Message(format!("Failed to query subscription quota: {e}")))?;

        if !quota.success {
            return Ok(UsageResult {
                success: false,
                data: None,
                error: quota.error.or(quota.credential_message),
            });
        }

        let data: Vec<UsageData> = quota
            .tiers
            .iter()
            .map(|tier| UsageData {
                plan_name: Some(tier.name.clone()),
                remaining: Some(100.0 - tier.utilization),
                total: Some(100.0),
                used: Some(tier.utilization),
                unit: Some("%".to_string()),
                is_valid: Some(true),
                invalid_message: None,
                extra: tier.resets_at.clone(),
            })
            .collect();

        return Ok(UsageResult {
            success: true,
            data: if data.is_empty() { None } else { Some(data) },
            error: None,
        });
    }

    // ── 通用 JS 脚本路径 ──
    crate::services::ProviderService::query_usage(state, app_type, provider_id)
        .await
        .map_err(|e| AppError::Message(e.to_string()))
}
