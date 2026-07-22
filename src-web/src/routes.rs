use axum::{
    body::Bytes,
    extract::{Json, Multipart, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use cc_switch_core::platform::Platform;
use cc_switch_core::store::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct InvokeRequest {
    pub cmd: String,
    #[serde(default)]
    pub args: Value,
}

#[derive(Debug, Serialize)]
pub struct InvokeResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
}

pub fn router(platform: Arc<dyn Platform>, app_state: AppState) -> Router {
    Router::new()
        .route("/api/invoke", post(invoke_handler))
        .route("/api/version", get(version_handler))
        .route("/api/info", get(info_handler))
        .route("/api/upload", post(upload_handler))
        .route("/api/download/:filename", get(download_handler))
        .route("/api/restart", post(restart_handler))
        .fallback_service(tower_http::services::ServeDir::new("dist-web"))
        .layer(Extension(platform))
        .layer(Extension(app_state))
}

async fn invoke_handler(
    Extension(platform): Extension<Arc<dyn Platform>>,
    Extension(app_state): Extension<AppState>,
    Json(req): Json<InvokeRequest>,
) -> Response {
    log::info!("invoke: {}", req.cmd);

    let db = app_state.db.clone();
    let proxy_service = app_state.proxy_service.clone();

    let result = match req.cmd.as_str() {
        "get_settings" => match cc_switch_core::commands::settings::get_settings() {
            Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
            Err(e) => Err(e.to_string()),
        },
        "save_settings" => match req.args.get("settings") {
            Some(v) => match serde_json::from_value::<cc_switch_core::settings::AppSettings>(v.clone()) {
                Ok(settings) => match cc_switch_core::commands::settings::save_settings(
                    &app_state,
                    settings,
                    &cc_switch_core::commands::settings::NoOpCodexHistoryMigrationHook,
                ) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            },
            None => Err("missing settings".to_string()),
        },
        "get_providers" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::provider::get_providers(&db, &app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_current_provider" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::provider::get_current_provider_id(&db, &app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "is_providers_empty" => match cc_switch_core::commands::provider::is_providers_empty(&db) {
            Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
            Err(e) => Err(e.to_string()),
        },
        "init_default_official_providers" => {
            match cc_switch_core::commands::provider::init_default_official_providers(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "save_provider" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let original_id = req
                .args
                .get("originalId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            match req.args.get("provider") {
                Some(v) => match serde_json::from_value::<cc_switch_core::provider::Provider>(v.clone()) {
                    Ok(provider) => match cc_switch_core::commands::provider::save_provider(&app_state, &app, provider, original_id.as_deref()) {
                        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                None => Err("missing provider".to_string()),
            }
        }
        "delete_provider" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::delete_provider(&app_state, &app, &id) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "switch_provider" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::switch_provider(&app_state, &app, &id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_custom_endpoints" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::get_custom_endpoints(&app_state, &app, &provider_id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "add_custom_endpoint" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let url = req
                .args
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::add_custom_endpoint(&app_state, &app, &provider_id, &url) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "remove_custom_endpoint" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let url = req
                .args
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::remove_custom_endpoint(&app_state, &app, &provider_id, &url) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_endpoint_last_used" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let url = req
                .args
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::update_endpoint_last_used(&app_state, &app, &provider_id, &url) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_providers_sort_order" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let updates = req
                .args
                .get("updates")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect()
                })
                .unwrap_or_default();
            match cc_switch_core::commands::provider::update_providers_sort_order(&app_state, &app, updates) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "import_default_config" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::provider::import_default_config(&app_state, &app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_universal_providers" => {
            match cc_switch_core::commands::provider::get_universal_providers(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_universal_provider" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::get_universal_provider(&app_state, &id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "upsert_universal_provider" => {
            match req.args.get("provider") {
                Some(v) => match serde_json::from_value::<cc_switch_core::provider::UniversalProvider>(v.clone()) {
                    Ok(provider) => match cc_switch_core::commands::provider::upsert_universal_provider(&app_state, provider) {
                        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                None => Err("missing provider".to_string()),
            }
        }
        "delete_universal_provider" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::delete_universal_provider(&app_state, &id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "sync_universal_provider" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::provider::sync_universal_provider(&app_state, &id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "import_opencode_providers_from_live" => {
            match cc_switch_core::commands::provider::import_opencode_providers_from_live(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_opencode_live_provider_ids" => {
            match cc_switch_core::commands::provider::get_opencode_live_provider_ids() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "add_provider" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let provider_value = req.args.get("provider").cloned().unwrap_or(Value::Null);
            let add_to_live = req.args.get("addToLive").and_then(|v| v.as_bool());
            match serde_json::from_value::<cc_switch_core::provider::Provider>(provider_value) {
                Ok(provider) => match cc_switch_core::commands::provider::add_provider(
                    &app_state,
                    &app,
                    provider,
                    add_to_live,
                ) {
                    Ok(v) => Ok(Value::Bool(v)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "update_provider" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let provider_value = req.args.get("provider").cloned().unwrap_or(Value::Null);
            let original_id = req.args.get("originalId").and_then(|v| v.as_str()).map(|s| s.to_string());
            match serde_json::from_value::<cc_switch_core::provider::Provider>(provider_value) {
                Ok(provider) => match cc_switch_core::commands::provider::update_provider(
                    &app_state,
                    &app,
                    provider,
                    original_id.as_deref(),
                ) {
                    Ok(v) => Ok(Value::Bool(v)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "remove_provider_from_live_config" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let id = req.args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::provider::remove_provider_from_live_config(&app_state, &app, &id) {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "read_live_provider_settings" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            match cc_switch_core::commands::provider::read_live_provider_settings(&app) {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string()),
            }
        }
        "testUsageScript" => {
            let provider_id = req.args.get("providerId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let script_code = req.args.get("scriptCode").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let timeout = req.args.get("timeout").and_then(|v| v.as_u64());
            let api_key = req.args.get("apiKey").and_then(|v| v.as_str()).map(|s| s.to_string());
            let base_url = req.args.get("baseUrl").and_then(|v| v.as_str()).map(|s| s.to_string());
            let access_token = req.args.get("accessToken").and_then(|v| v.as_str()).map(|s| s.to_string());
            let user_id = req.args.get("userId").and_then(|v| v.as_str()).map(|s| s.to_string());
            let template_type = req.args.get("templateType").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::provider::test_usage_script(
                &app_state,
                &provider_id,
                &app,
                &script_code,
                timeout,
                api_key.as_deref(),
                base_url.as_deref(),
                access_token.as_deref(),
                user_id.as_deref(),
                template_type.as_deref(),
            )
            .await
            {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "ensure_claude_desktop_official_provider" => {
            match cc_switch_core::commands::provider::ensure_claude_desktop_official_provider(&db) {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "ensure_codex_official_provider" => {
            match cc_switch_core::commands::provider::ensure_codex_official_provider(&db) {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_claude_desktop_default_routes" => {
            Ok(serde_json::to_value(cc_switch_core::commands::provider::get_claude_desktop_default_routes())
                .unwrap_or(Value::Null))
        }
        "get_claude_desktop_status" => {
            match cc_switch_core::commands::provider::get_claude_desktop_status(&app_state).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "import_claude_desktop_providers_from_claude" => {
            match cc_switch_core::commands::provider::import_claude_desktop_providers_from_claude(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "list_profiles" => {
            match cc_switch_core::commands::profile::get_profiles(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- Settings 配置读写命令（10 个 A 类）-----
        "get_app_config_dir_override" => {
            Ok(serde_json::to_value(cc_switch_core::commands::settings::get_app_config_dir_override())
                .unwrap_or(Value::Null))
        }
        "set_app_config_dir_override" => {
            let path = req.args.get("path").and_then(|v| v.as_str());
            cc_switch_core::commands::settings::set_app_config_dir_override(path);
            Ok(Value::Bool(true))
        }
        "get_rectifier_config" => {
            match cc_switch_core::commands::settings::get_rectifier_config(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_rectifier_config" => {
            let config_value = req.args.get("config").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::proxy::RectifierConfig>(config_value) {
                Ok(config) => match cc_switch_core::commands::settings::set_rectifier_config(&app_state, config) {
                    Ok(v) => Ok(Value::Bool(v)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_optimizer_config" => {
            match cc_switch_core::commands::settings::get_optimizer_config(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_optimizer_config" => {
            let config_value = req.args.get("config").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::proxy::OptimizerConfig>(config_value) {
                Ok(config) => match cc_switch_core::commands::settings::set_optimizer_config(&app_state, config) {
                    Ok(v) => Ok(Value::Bool(v)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_copilot_optimizer_config" => {
            match cc_switch_core::commands::settings::get_copilot_optimizer_config(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_copilot_optimizer_config" => {
            let config_value = req.args.get("config").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::proxy::CopilotOptimizerConfig>(config_value) {
                Ok(config) => match cc_switch_core::commands::settings::set_copilot_optimizer_config(&app_state, config) {
                    Ok(v) => Ok(Value::Bool(v)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_log_config" => {
            match cc_switch_core::commands::settings::get_log_config(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_log_config" => {
            let config_value = req.args.get("config").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::proxy::LogConfig>(config_value) {
                Ok(config) => match cc_switch_core::commands::settings::set_log_config(&app_state, config) {
                    Ok(v) => Ok(Value::Bool(v)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "set_auto_launch" => {
            let enabled = req.args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            match cc_switch_core::commands::settings::set_auto_launch(enabled) {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_auto_launch_status" => {
            match cc_switch_core::commands::settings::get_auto_launch_status() {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "has_codex_unify_history_backup" => {
            Ok(Value::Bool(cc_switch_core::commands::settings::has_codex_unify_history_backup()))
        }
        "restore_codex_unified_history" => {
            match tokio::task::spawn_blocking(|| {
                cc_switch_core::commands::settings::restore_codex_unified_history()
            })
            .await
            {
                Ok(Ok(v)) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- Claude 插件命令（6 个 A 类）-----
        "get_claude_plugin_status" => {
            match cc_switch_core::commands::plugin::get_claude_plugin_status() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "read_claude_plugin_config" => {
            match cc_switch_core::commands::plugin::read_claude_plugin_config() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "apply_claude_plugin_config" => {
            let official = req.args.get("official").and_then(|v| v.as_bool()).unwrap_or(false);
            match cc_switch_core::commands::plugin::apply_claude_plugin_config(official) {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "is_claude_plugin_applied" => {
            match cc_switch_core::commands::plugin::is_claude_plugin_applied() {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "apply_claude_onboarding_skip" => {
            match cc_switch_core::commands::plugin::apply_claude_onboarding_skip() {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "clear_claude_onboarding_skip" => {
            match cc_switch_core::commands::plugin::clear_claude_onboarding_skip() {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- Deep-link 导入命令（4 个 A 类）-----
        "parse_deeplink" => {
            let url = req.args.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::deeplink::parse_deeplink(&url) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "merge_deeplink_config" => {
            let request_value = req.args.get("request").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::deeplink::DeepLinkImportRequest>(request_value) {
                Ok(request) => match cc_switch_core::commands::deeplink::merge_deeplink_config(request) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "import_from_deeplink" => {
            let request_value = req.args.get("request").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::deeplink::DeepLinkImportRequest>(request_value) {
                Ok(request) => match cc_switch_core::commands::deeplink::import_from_deeplink(&app_state, request) {
                    Ok(v) => Ok(Value::String(v)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "import_from_deeplink_unified" => {
            let request_value = req.args.get("request").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::deeplink::DeepLinkImportRequest>(request_value) {
                Ok(request) => match cc_switch_core::commands::deeplink::import_from_deeplink_unified(&app_state, request).await {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- Skill 本地兼容命令（3 个 A 类）-----
        "restore_skill_backup" => {
            let backup_id = req.args.get("backupId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let current_app = req.args.get("currentApp").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            match cc_switch_core::commands::skill::restore_skill_backup(&db, &backup_id, &current_app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "uninstall_skill" => {
            let directory = req.args.get("directory").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::skill::uninstall_skill(&db, &directory) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "uninstall_skill_for_app" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let directory = req.args.get("directory").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::skill::uninstall_skill_for_app(&db, &app, &directory) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- SkillService 网络方法（9 个，B 类已下沉）-----
        "get_skills" => {
            match cc_switch_core::commands::skill::get_skills(&db).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_skills_for_app" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            match cc_switch_core::commands::skill::get_skills_for_app(&db, &app).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "discover_available_skills" => {
            match cc_switch_core::commands::skill::discover_available_skills(&db).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "install_skill_unified" => {
            let skill_value = req.args.get("skill").cloned().unwrap_or(Value::Null);
            let current_app = req.args.get("currentApp").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            match serde_json::from_value::<cc_switch_core::services::skill::DiscoverableSkill>(skill_value) {
                Ok(skill) => match cc_switch_core::commands::skill::install_skill_unified(&db, skill, &current_app).await {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "install_skill" => {
            let directory = req.args.get("directory").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::skill::install_skill(&db, &directory).await {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "install_skill_for_app" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let directory = req.args.get("directory").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::skill::install_skill_for_app(&db, &app, &directory).await {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "check_skill_updates" => {
            match cc_switch_core::commands::skill::check_skill_updates(&db).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_skill" => {
            let id = req.args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::skill::update_skill(&db, &id).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "search_skills_sh" => {
            let query = req.args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let limit = req.args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            let offset = req.args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            match cc_switch_core::commands::skill::search_skills_sh(&query, limit, offset).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- Copilot OAuth 命令（8 个，B 类已下沉）-----
        "copilot_start_device_flow" => {
            let github_domain = req.args.get("githubDomain").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::copilot::copilot_start_device_flow(&app_state, github_domain.as_deref()).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "copilot_list_accounts" => {
            match cc_switch_core::commands::copilot::copilot_list_accounts(&app_state).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "copilot_get_auth_status" => {
            match cc_switch_core::commands::copilot::copilot_get_auth_status(&app_state).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "copilot_is_authenticated" => {
            match cc_switch_core::commands::copilot::copilot_is_authenticated(&app_state).await {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "copilot_logout" => {
            match cc_switch_core::commands::copilot::copilot_logout(&app_state).await {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "copilot_get_token" => {
            match cc_switch_core::commands::copilot::copilot_get_token(&app_state).await {
                Ok(v) => Ok(Value::String(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "copilot_get_models" => {
            match cc_switch_core::commands::copilot::copilot_get_models(&app_state).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "copilot_get_usage" => {
            match cc_switch_core::commands::copilot::copilot_get_usage(&app_state).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- C 类：reset_circuit_breaker（事件拆分完成）-----
        "reset_circuit_breaker" => {
            let provider_id = req.args.get("providerId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let app_type = req.args.get("appType").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::proxy::reset_circuit_breaker(&app_state, &provider_id, &app_type).await {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- C 类：queryProviderUsage（事件拆分完成）-----
        "queryProviderUsage" => {
            let provider_id = req.args.get("providerId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            match app.parse::<cc_switch_core::app_config::AppType>() {
                Ok(app_type) => match cc_switch_core::commands::provider::query_provider_usage(&app_state, &provider_id, &app).await {
                    Ok(snapshot) => {
                        // C 类副作用：发射 usage-cache-updated 事件。
                        // core 不发射事件；外壳（Web）负责 SSE 广播。
                        let payload = serde_json::json!({
                            "kind": "script",
                            "appType": app_type.as_str(),
                            "providerId": &provider_id,
                            "data": &snapshot,
                        });
                        platform.emit_event("usage-cache-updated", payload);
                        Ok(serde_json::to_value(snapshot).unwrap_or(Value::Null))
                    }
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- D 类：系统 GUI/桌面集成命令（Web 端兜底）-----
        // 这些命令在桌面版依赖 Tauri 的窗口/托盘/对话框/opener/updater 等能力，
        // 在无头 Web 模式下无法实现，统一返回明确错误，前端隐藏对应 UI。
        // ----- D 类：系统 GUI/桌面集成命令（Web 端兜底）-----
        // 这些命令在桌面版依赖 Tauri 的窗口/托盘/opener/updater 等能力，
        // 在无头 Web 模式下无法实现，统一返回明确错误，前端隐藏对应 UI。
        // 注：文件对话框（open_file_dialog / save_file_dialog /
        // open_zip_file_dialog / pick_directory）已在前端 core.ts shim 中
        // 用 HTML <input> 拦截，不会走到这里；restart_app 走 /api/restart；
        // open_hermes_web_ui 已迁移到 core 真实实现（返回 URL 给前端）；
        // check_app_update_available 已迁移到 core（HTTP 查询 GitHub releases）；
        // set_window_theme 在 Web 模式下 no-op 成功。
        // is_lightweight_mode 在 Web 模式下永远返回 false（非轻量模式）。
        // P4-B 已迁移：open_app_config_folder / open_config_folder /
        // open_workspace_directory（返回路径）/ open_provider_terminal /
        // launch_session_terminal（返回命令字符串）。
        "is_lightweight_mode" => Ok(Value::Bool(false)),
        "open_app_config_folder" => {
            match cc_switch_core::commands::config::open_app_config_folder() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "open_config_folder" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            match cc_switch_core::commands::config::open_config_folder(&app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "open_workspace_directory" => {
            let subdir = req.args.get("subdir").and_then(|v| v.as_str()).unwrap_or("workspace").to_string();
            match cc_switch_core::commands::config::open_workspace_directory(&subdir) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "open_provider_terminal" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let provider_id = req.args.get("providerId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let cwd = req.args.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::misc::open_provider_terminal(&app_state, &app, &provider_id, cwd.as_deref()).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "launch_session_terminal" => {
            let command = req.args.get("command").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let cwd = req.args.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::misc::launch_session_terminal(&command, cwd.as_deref()) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "install_update_and_restart"
        | "launch_hermes_dashboard"
        | "enter_lightweight_mode"
        | "exit_lightweight_mode" => Err(format!(
            "Command '{}' is not supported in headless Web mode. Use the desktop application.",
            req.cmd
        )),
        // ----- D 类降级：set_window_theme（Web 模式 no-op）-----
        "set_window_theme" => {
            let theme = req.args.get("theme").and_then(|v| v.as_str()).unwrap_or("system").to_string();
            match cc_switch_core::commands::misc::set_window_theme(&theme) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- D 类降级：check_app_update_available（HTTP 查询 GitHub releases）-----
        "check_app_update_available" => {
            match cc_switch_core::commands::settings::check_app_update_available().await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- D 类降级：open_hermes_web_ui（返回 URL 给前端 window.open）-----
        "open_hermes_web_ui" => {
            let path = req.args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::hermes::open_hermes_web_ui(path.as_deref()).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- E 类：环境变量管理（已下沉，需用户明确知道修改的是服务器文件）-----
        "check_env_conflicts" => {
            let app = req.args.get("app").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::env::check_env_conflicts(&app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_env_vars" => {
            let conflicts_value = req.args.get("conflicts").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<Vec<cc_switch_core::services::env_checker::EnvConflict>>(conflicts_value) {
                Ok(conflicts) => match cc_switch_core::commands::env::delete_env_vars(conflicts) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "restore_env_backup" => {
            let backup_path = req.args.get("backupPath").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::env::restore_env_backup(&backup_path) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- E 类：工具版本/安装探测（简化版实现）-----
        "get_tool_versions" => {
            let tools = req.args.get("tools").and_then(|v| v.as_array()).map(|a| {
                a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>()
            });
            match cc_switch_core::commands::misc::get_tool_versions(tools).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "probe_tool_installations" => {
            let tools_value = req.args.get("tools").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<Vec<String>>(tools_value) {
                Ok(tools) => match cc_switch_core::commands::misc::probe_tool_installations(tools).await {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "run_tool_lifecycle_action" => {
            let tools_value = req.args.get("tools").cloned().unwrap_or(Value::Null);
            let action = req.args.get("action").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match serde_json::from_value::<Vec<String>>(tools_value) {
                Ok(tools) => match cc_switch_core::commands::misc::run_tool_lifecycle_action(tools, action).await {
                    Ok(()) => Ok(Value::Bool(true)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- 额外：stream_check + coding_plan（B 类已下沉完成）-----
        "stream_check_provider" => {
            let app = req.args.get("appType").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let provider_id = req.args.get("providerId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match app.parse::<cc_switch_core::app_config::AppType>() {
                Ok(app_type) => match cc_switch_core::commands::stream_check::stream_check_provider(&app_state, app_type, &provider_id).await {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "stream_check_all_providers" => {
            let app = req.args.get("appType").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let proxy_targets_only = req.args.get("proxyTargetsOnly").and_then(|v| v.as_bool()).unwrap_or(false);
            match app.parse::<cc_switch_core::app_config::AppType>() {
                Ok(app_type) => match cc_switch_core::commands::stream_check::stream_check_all_providers(&app_state, app_type, proxy_targets_only).await {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_coding_plan_quota" => {
            let base_url = req.args.get("baseUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let api_key = req.args.get("apiKey").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let access_key_id = req.args.get("accessKeyId").and_then(|v| v.as_str()).map(|s| s.to_string());
            let secret_access_key = req.args.get("secretAccessKey").and_then(|v| v.as_str()).map(|s| s.to_string());
            let coding_plan_provider = req.args.get("codingPlanProvider").and_then(|v| v.as_str()).map(|s| s.to_string());
            let team_organization_id = req.args.get("teamOrganizationId").and_then(|v| v.as_str()).map(|s| s.to_string());
            let team_project_id = req.args.get("teamProjectId").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::coding_plan::get_coding_plan_quota(
                &base_url,
                &api_key,
                access_key_id.as_deref(),
                secret_access_key.as_deref(),
                coding_plan_provider.as_deref(),
                team_organization_id.as_deref(),
                team_project_id.as_deref(),
            ).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- Session manager 命令（6 个）-----
        "list_sessions" => {
            match tokio::task::spawn_blocking(|| {
                cc_switch_core::commands::session_manager::list_sessions()
            })
            .await
            {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_session_messages" => {
            let provider_id = req.args.get("providerId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let source_path = req.args.get("sourcePath").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::session_manager::get_session_messages(&provider_id, &source_path)
            })
            .await
            {
                Ok(Ok(v)) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_session" => {
            let provider_id = req.args.get("providerId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let session_id = req.args.get("sessionId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let source_path = req.args.get("sourcePath").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::session_manager::delete_session(&provider_id, &session_id, &source_path)
            })
            .await
            {
                Ok(Ok(v)) => Ok(Value::Bool(v)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_sessions" => {
            let items_value = req.args.get("items").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<Vec<cc_switch_core::session_manager::DeleteSessionRequest>>(items_value) {
                Ok(items) => {
                    match tokio::task::spawn_blocking(move || {
                        cc_switch_core::commands::session_manager::delete_sessions(items)
                    })
                    .await
                    {
                        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                        Err(e) => Err(e.to_string()),
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        }
        "sync_session_usage" => {
            match cc_switch_core::commands::session_manager::sync_session_usage(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_usage_data_sources" => {
            match cc_switch_core::commands::session_manager::get_usage_data_sources(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- Usage stats 命令（11 个）-----
        "get_usage_summary" => {
            let start_date = req.args.get("startDate").and_then(|v| v.as_i64());
            let end_date = req.args.get("endDate").and_then(|v| v.as_i64());
            let app_type = req.args.get("appType").and_then(|v| v.as_str()).map(|s| s.to_string());
            let provider_name = req.args.get("providerName").and_then(|v| v.as_str()).map(|s| s.to_string());
            let model = req.args.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::usage::get_usage_summary(
                &db,
                start_date,
                end_date,
                app_type.as_deref(),
                provider_name.as_deref(),
                model.as_deref(),
            ) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_usage_summary_by_app" => {
            let start_date = req.args.get("startDate").and_then(|v| v.as_i64());
            let end_date = req.args.get("endDate").and_then(|v| v.as_i64());
            let provider_name = req.args.get("providerName").and_then(|v| v.as_str()).map(|s| s.to_string());
            let model = req.args.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::usage::get_usage_summary_by_app(
                &db,
                start_date,
                end_date,
                provider_name.as_deref(),
                model.as_deref(),
            ) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_usage_trends" => {
            let start_date = req.args.get("startDate").and_then(|v| v.as_i64());
            let end_date = req.args.get("endDate").and_then(|v| v.as_i64());
            let app_type = req.args.get("appType").and_then(|v| v.as_str()).map(|s| s.to_string());
            let provider_name = req.args.get("providerName").and_then(|v| v.as_str()).map(|s| s.to_string());
            let model = req.args.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::usage::get_usage_trends(
                &db,
                start_date,
                end_date,
                app_type.as_deref(),
                provider_name.as_deref(),
                model.as_deref(),
            ) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_provider_stats" => {
            let start_date = req.args.get("startDate").and_then(|v| v.as_i64());
            let end_date = req.args.get("endDate").and_then(|v| v.as_i64());
            let app_type = req.args.get("appType").and_then(|v| v.as_str()).map(|s| s.to_string());
            let provider_name = req.args.get("providerName").and_then(|v| v.as_str()).map(|s| s.to_string());
            let model = req.args.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::usage::get_provider_stats(
                &db,
                start_date,
                end_date,
                app_type.as_deref(),
                provider_name.as_deref(),
                model.as_deref(),
            ) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_model_stats" => {
            let start_date = req.args.get("startDate").and_then(|v| v.as_i64());
            let end_date = req.args.get("endDate").and_then(|v| v.as_i64());
            let app_type = req.args.get("appType").and_then(|v| v.as_str()).map(|s| s.to_string());
            let provider_name = req.args.get("providerName").and_then(|v| v.as_str()).map(|s| s.to_string());
            let model = req.args.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
            match cc_switch_core::commands::usage::get_model_stats(
                &db,
                start_date,
                end_date,
                app_type.as_deref(),
                provider_name.as_deref(),
                model.as_deref(),
            ) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_request_logs" => {
            let filters_value = req.args.get("filters").cloned().unwrap_or(Value::Null);
            let page = req.args.get("page").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let page_size = req.args.get("pageSize").and_then(|v| v.as_u64()).unwrap_or(20) as u32;
            match serde_json::from_value::<cc_switch_core::services::usage_stats::LogFilters>(filters_value) {
                Ok(filters) => match cc_switch_core::commands::usage::get_request_logs(&db, &filters, page, page_size) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_request_detail" => {
            let request_id = req.args.get("requestId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::usage::get_request_detail(&db, &request_id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_model_pricing" => {
            match cc_switch_core::commands::usage::get_model_pricing(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_model_pricing" => {
            let model_id = req.args.get("modelId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let display_name = req.args.get("displayName").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let input_cost = req.args.get("inputCost").and_then(|v| v.as_str()).unwrap_or("0").to_string();
            let output_cost = req.args.get("outputCost").and_then(|v| v.as_str()).unwrap_or("0").to_string();
            let cache_read_cost = req.args.get("cacheReadCost").and_then(|v| v.as_str()).unwrap_or("0").to_string();
            let cache_creation_cost = req.args.get("cacheCreationCost").and_then(|v| v.as_str()).unwrap_or("0").to_string();
            match cc_switch_core::commands::usage::update_model_pricing(
                &db, &model_id, &display_name, &input_cost, &output_cost, &cache_read_cost, &cache_creation_cost,
            ) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "check_provider_limits" => {
            let provider_id = req.args.get("providerId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let app_type = req.args.get("appType").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::usage::check_provider_limits(&db, &provider_id, &app_type) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_model_pricing" => {
            let model_id = req.args.get("modelId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match cc_switch_core::commands::usage::delete_model_pricing(&db, &model_id) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_mcp_servers" => {
            match cc_switch_core::commands::mcp::get_mcp_servers(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "upsert_mcp_server" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let spec = req
                .args
                .get("spec")
                .cloned()
                .unwrap_or(Value::Null);
            let apps = req
                .args
                .get("apps")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            match cc_switch_core::commands::mcp::upsert_mcp_server(&app_state, &id, spec, apps) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_mcp_server" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::mcp::delete_mcp_server(&app_state, &id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "toggle_mcp_app" => {
            let server_id = req
                .args
                .get("serverId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let enabled = req
                .args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match cc_switch_core::commands::mcp::toggle_mcp_app(&app_state, &server_id, &app, enabled) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_claude_mcp_status" => {
            match cc_switch_core::commands::mcp::get_claude_mcp_status() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "read_claude_mcp_config" => {
            match cc_switch_core::commands::mcp::read_claude_mcp_config() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "validate_mcp_command" => {
            let cmd = req
                .args
                .get("cmd")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::mcp::validate_mcp_command(&cmd) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "read_mcp_servers_map" => {
            match cc_switch_core::commands::mcp::read_mcp_servers_map() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "upsert_claude_mcp_server" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let spec = req
                .args
                .get("spec")
                .cloned()
                .unwrap_or(Value::Null);
            match cc_switch_core::commands::mcp::upsert_claude_mcp_server(&id, spec) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_claude_mcp_server" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::mcp::delete_claude_mcp_server(&id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_prompts" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::prompt::get_prompts(&app_state, &app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "upsert_prompt" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match req.args.get("prompt") {
                Some(v) => match serde_json::from_value::<cc_switch_core::prompt::Prompt>(v.clone()) {
                    Ok(prompt) => match cc_switch_core::commands::prompt::upsert_prompt(&app_state, &app, &id, prompt) {
                        Ok(()) => Ok(Value::Bool(true)),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                None => Err("missing prompt".to_string()),
            }
        }
        "delete_prompt" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::prompt::delete_prompt(&app_state, &app, &id) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "enable_prompt" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::prompt::enable_prompt(&app_state, &app, &id) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "import_prompt_from_file" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::prompt::import_prompt_from_file(&app_state, &app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_current_prompt_file_content" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::prompt::get_current_prompt_file_content(&app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_profiles" => {
            match cc_switch_core::commands::profile::get_profiles(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "create_profile" => {
            let name = req
                .args
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let scope = req
                .args
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::profile::create_profile(&app_state, &name, &scope) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_profile" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::profile::delete_profile(&app_state, &id) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_profile" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = req
                .args
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let resnapshot = req
                .args
                .get("resnapshot")
                .and_then(|v| v.as_bool());
            let scope = req
                .args
                .get("scope")
                .and_then(|v| v.as_str());
            match cc_switch_core::commands::profile::update_profile(
                &app_state,
                &id,
                name,
                resnapshot,
                scope,
            ) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "clear_current_profile" => {
            let scope = req
                .args
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::profile::clear_current_profile(&app_state, &scope) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "apply_profile" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let scope = req
                .args
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::profile::apply_profile(&app_state, &id, &scope) {
                Ok((warnings, should_stop_proxy)) => Ok(serde_json::to_value(serde_json::json!({
                    "warnings": warnings,
                    "shouldStopProxy": should_stop_proxy,
                })).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_config_status" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let proxy_running = proxy_service.is_running().await;
            match cc_switch_core::commands::config::get_config_status(&db, &app, proxy_running) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_config_dir" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::config::get_config_dir(&app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_claude_code_config_path" => {
            match cc_switch_core::commands::config::get_claude_code_config_path() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_app_config_path" => {
            match cc_switch_core::commands::config::get_app_config_path() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_config_snippet" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::config::get_config_snippet(&db, &app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_config_snippet" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let snippet = req
                .args
                .get("snippet")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            match cc_switch_core::commands::config::set_config_snippet(&db, &app, snippet) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "clear_config_snippet" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::config::clear_config_snippet(&db, &app) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_balance" => {
            let base_url = req
                .args
                .get("baseUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let api_key = req
                .args
                .get("apiKey")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::balance::get_balance(&base_url, &api_key).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_subscription_quota" => {
            let tool = req
                .args
                .get("tool")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::subscription::get_subscription_quota(&tool).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "test_api_endpoints" => {
            let urls = req
                .args
                .get("urls")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let timeout_secs = req
                .args
                .get("timeoutSecs")
                .and_then(|v| v.as_u64());
            match cc_switch_core::commands::speedtest::test_api_endpoints(urls, timeout_secs).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_installed_skills" => {
            match cc_switch_core::commands::skill::get_installed_skills(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_skill_backups" => {
            match cc_switch_core::commands::skill::get_skill_backups() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_skill_backup" => {
            let backup_id = req
                .args
                .get("backupId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::skill::delete_skill_backup(&backup_id) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "uninstall_skill_unified" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::skill::uninstall_skill_unified(&db, &id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "toggle_skill_app" => {
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let enabled = req
                .args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match cc_switch_core::commands::skill::toggle_skill_app(&db, &id, &app, enabled) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "scan_unmanaged_skills" => {
            match cc_switch_core::commands::skill::scan_unmanaged_skills(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "import_skills_from_apps" => {
            let imports = req
                .args
                .get("imports")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect()
                })
                .unwrap_or_default();
            match cc_switch_core::commands::skill::import_skills_from_apps(&db, imports) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_skill_repos" => {
            match cc_switch_core::commands::skill::get_skill_repos(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "add_skill_repo" => match req.args.get("repo") {
            Some(v) => match serde_json::from_value::<cc_switch_core::services::skill::SkillRepo>(v.clone()) {
                Ok(repo) => match cc_switch_core::commands::skill::add_skill_repo(&db, repo) {
                    Ok(()) => Ok(Value::Bool(true)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            },
            None => Err("missing repo".to_string()),
        }
        "remove_skill_repo" => {
            let owner = req
                .args
                .get("owner")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = req
                .args
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::skill::remove_skill_repo(&db, &owner, &name) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "install_skills_from_zip" => {
            let file_path = req
                .args
                .get("filePath")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let current_app = req
                .args
                .get("currentApp")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::skill::install_skills_from_zip(&db, &file_path, &current_app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "migrate_skill_storage" => match req.args.get("target") {
            Some(v) => match serde_json::from_value::<cc_switch_core::services::skill::SkillStorageLocation>(v.clone()) {
                Ok(target) => match cc_switch_core::commands::skill::migrate_skill_storage(&db, target) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            },
            None => Err("missing target".to_string()),
        }
        "get_stream_check_config" => {
            match cc_switch_core::commands::stream_check::get_stream_check_config(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "save_stream_check_config" => match req.args.get("config") {
            Some(v) => match serde_json::from_value::<cc_switch_core::services::stream_check::StreamCheckConfig>(v.clone()) {
                Ok(config) => match cc_switch_core::commands::stream_check::save_stream_check_config(&db, config) {
                    Ok(()) => Ok(Value::Bool(true)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            },
            None => Err("missing config".to_string()),
        }
        "get_global_proxy_url" => {
            match cc_switch_core::commands::global_proxy::get_global_proxy_url(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_global_proxy_url" => {
            let url = req
                .args
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::global_proxy::set_global_proxy_url(&db, &url) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "test_proxy_url" => {
            let url = req
                .args
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::global_proxy::test_proxy_url(&url).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_upstream_proxy_status" => {
            Ok(serde_json::to_value(cc_switch_core::commands::global_proxy::get_upstream_proxy_status()).unwrap_or(Value::Null))
        }
        "export_config_to_file" => {
            let file_path = req
                .args
                .get("filePath")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::import_export::export_config_to_file(&db, &file_path) {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string()),
            }
        }
        "import_config_from_file" => {
            let file_path = req
                .args
                .get("filePath")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let db = db.clone();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::import_export::import_config_from_file(&db, &file_path)
            })
            .await
            {
                Ok(Ok(v)) => Ok(v),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "sync_current_providers_live" => {
            match cc_switch_core::commands::import_export::sync_current_providers_live(&app_state) {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string()),
            }
        }
        "read_omo_local_file" => {
            match cc_switch_core::commands::omo::read_omo_local_file() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_current_omo_provider_id" => {
            match cc_switch_core::commands::omo::get_current_omo_provider_id(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "disable_current_omo" => {
            match cc_switch_core::commands::omo::disable_current_omo(&db) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "read_omo_slim_local_file" => {
            match cc_switch_core::commands::omo::read_omo_slim_local_file() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_current_omo_slim_provider_id" => {
            match cc_switch_core::commands::omo::get_current_omo_slim_provider_id(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "disable_current_omo_slim" => {
            match cc_switch_core::commands::omo::disable_current_omo_slim(&db) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_failover_queue" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::failover::get_failover_queue(&db, &app_type) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_available_providers_for_failover" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::failover::get_available_providers_for_failover(&db, &app_type) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "add_to_failover_queue" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::failover::add_to_failover_queue(&db, &app_type, &provider_id) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "remove_from_failover_queue" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::failover::remove_from_failover_queue(&db, &app_type, &provider_id) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_auto_failover_enabled" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::failover::get_auto_failover_enabled(&db, &app_type).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_auto_failover_enabled" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let enabled = req
                .args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match cc_switch_core::commands::failover::set_auto_failover_enabled(&app_state, &app_type, enabled).await {
                Ok(_) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_proxy_status" => {
            match cc_switch_core::commands::proxy::get_proxy_status(&proxy_service).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "start_proxy_server" => {
            match cc_switch_core::commands::proxy::start_proxy_server(&proxy_service).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "stop_proxy_server" => {
            match cc_switch_core::commands::proxy::stop_proxy_server(&proxy_service).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "stop_proxy_with_restore" => {
            match cc_switch_core::commands::proxy::stop_proxy_with_restore(&proxy_service).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_proxy_takeover_status" => {
            match cc_switch_core::commands::proxy::get_proxy_takeover_status(&proxy_service).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_proxy_takeover_for_app" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let enabled = req
                .args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match cc_switch_core::commands::proxy::set_proxy_takeover_for_app(&proxy_service, &app_type, enabled).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_proxy_config" => {
            match cc_switch_core::commands::proxy::get_proxy_config(&proxy_service).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_proxy_config" => {
            match req.args.get("config") {
                Some(v) => match serde_json::from_value::<cc_switch_core::proxy::ProxyConfig>(v.clone()) {
                    Ok(config) => match cc_switch_core::commands::proxy::update_proxy_config(&proxy_service, config).await {
                        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                None => Err("missing config".to_string()),
            }
        }
        "get_global_proxy_config" => {
            match cc_switch_core::commands::proxy::get_global_proxy_config(&app_state).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_global_proxy_config" => {
            match req.args.get("config") {
                Some(v) => match serde_json::from_value::<cc_switch_core::proxy::GlobalProxyConfig>(v.clone()) {
                    Ok(config) => match cc_switch_core::commands::proxy::update_global_proxy_config(&app_state, config).await {
                        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                None => Err("missing config".to_string()),
            }
        }
        "get_proxy_config_for_app" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::proxy::get_proxy_config_for_app(&app_state, &app_type).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_proxy_config_for_app" => {
            match req.args.get("config") {
                Some(v) => match serde_json::from_value::<cc_switch_core::proxy::AppProxyConfig>(v.clone()) {
                    Ok(config) => match cc_switch_core::commands::proxy::update_proxy_config_for_app(&app_state, config).await {
                        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                None => Err("missing config".to_string()),
            }
        }
        "get_default_cost_multiplier" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::proxy::get_default_cost_multiplier(&app_state, &app_type).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_default_cost_multiplier" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let value = req
                .args
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::proxy::set_default_cost_multiplier(&app_state, &app_type, &value).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_pricing_model_source" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::proxy::get_pricing_model_source(&app_state, &app_type).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_pricing_model_source" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let value = req
                .args
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::proxy::set_pricing_model_source(&app_state, &app_type, &value).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "is_proxy_running" => {
            Ok(serde_json::to_value(cc_switch_core::commands::proxy::is_proxy_running(&proxy_service).await).unwrap_or(Value::Null))
        }
        "is_live_takeover_active" => {
            match cc_switch_core::commands::proxy::is_live_takeover_active(&proxy_service).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "fetch_models_for_config" => {
            let base_url = req.args.get("baseUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let api_key = req.args.get("apiKey").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let is_full_url = req.args.get("isFullUrl").and_then(|v| v.as_bool()).unwrap_or(false);
            let models_url = req.args.get("modelsUrl").and_then(|v| v.as_str()).map(|s| s.to_string());
            let custom_user_agent = req.args.get("customUserAgent").and_then(|v| v.as_str()).map(|s| s.to_string());

            if api_key.is_empty() {
                Err("API Key is required to fetch models".to_string())
            } else {
                let candidates = if let Some(ref override_url) = models_url {
                    let trimmed = override_url.trim();
                    if !trimmed.is_empty() {
                        vec![trimmed.to_string()]
                    } else {
                        let trimmed = base_url.trim().trim_end_matches('/');
                        if trimmed.is_empty() {
                            vec![]
                        } else if is_full_url {
                            if let Some(idx) = trimmed.find("/v1/") {
                                vec![format!("{}/v1/models", &trimmed[..idx])]
                            } else {
                                vec![format!("{}/v1/models", trimmed)]
                            }
                        } else {
                            vec![format!("{}/v1/models", trimmed)]
                        }
                    }
                } else {
                    let trimmed = base_url.trim().trim_end_matches('/');
                    if trimmed.is_empty() {
                        vec![]
                    } else if is_full_url {
                        if let Some(idx) = trimmed.find("/v1/") {
                            vec![format!("{}/v1/models", &trimmed[..idx])]
                        } else {
                            vec![format!("{}/v1/models", trimmed)]
                        }
                    } else {
                        vec![format!("{}/v1/models", trimmed)]
                    }
                };

                if candidates.is_empty() {
                    Err("Base URL is empty".to_string())
                } else {
                    let client = cc_switch_core::proxy::http_client::get();
                    let mut last_err: Option<String> = None;
                    let mut result: Option<serde_json::Value> = None;

                    for url in &candidates {
                        let mut request = client
                            .get(url)
                            .header("Authorization", format!("Bearer {}", api_key))
                            .timeout(std::time::Duration::from_secs(15));
                        if let Some(ua) = &custom_user_agent {
                            if let Ok(hv) = reqwest::header::HeaderValue::from_str(ua) {
                                request = request.header(reqwest::header::USER_AGENT, hv);
                            }
                        }
                        match request.send().await {
                            Ok(response) => {
                                let status = response.status();
                                if status.is_success() {
                                    match response.json::<serde_json::Value>().await {
                                        Ok(json) => {
                                            let models = json.get("data").and_then(|d| d.as_array())
                                                .map(|arr| {
                                                    arr.iter().filter_map(|m| {
                                                        let id = m.get("id").and_then(|v| v.as_str())?;
                                                        Some(serde_json::json!({
                                                            "id": id,
                                                            "ownedBy": m.get("owned_by").and_then(|v| v.as_str())
                                                        }))
                                                    }).collect::<Vec<_>>()
                                                })
                                                .unwrap_or_default();
                                            result = Some(serde_json::Value::Array(models));
                                            break;
                                        }
                                        Err(e) => {
                                            last_err = Some(format!("Failed to parse response: {e}"));
                                            continue;
                                        }
                                    }
                                } else if status == reqwest::StatusCode::NOT_FOUND || status == reqwest::StatusCode::METHOD_NOT_ALLOWED {
                                    let body = response.text().await.unwrap_or_default();
                                    let truncated: String = body.chars().take(512).collect();
                                    last_err = Some(format!("HTTP {status}: {truncated}"));
                                    continue;
                                } else {
                                    let body = response.text().await.unwrap_or_default();
                                    let truncated: String = body.chars().take(512).collect();
                                    last_err = Some(format!("HTTP {status}: {truncated}"));
                                    break;
                                }
                            }
                            Err(e) => {
                                last_err = Some(format!("Request failed: {e}"));
                                break;
                            }
                        }
                    }

                    match result {
                        Some(models) => Ok(models),
                        None => Err(format!("All candidates failed: {}", last_err.unwrap_or_else(|| "no candidates".to_string()))),
                    }
                }
            }
        }
        "switch_proxy_provider" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::proxy::switch_proxy_provider(&app_state, &app_type, &provider_id).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_provider_health" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::proxy::get_provider_health(&app_state, &provider_id, &app_type).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_circuit_breaker_config" => {
            match cc_switch_core::commands::proxy::get_circuit_breaker_config(&app_state).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_circuit_breaker_config" => {
            match req.args.get("config") {
                Some(v) => match serde_json::from_value::<cc_switch_core::proxy::CircuitBreakerConfig>(v.clone()) {
                    Ok(config) => match cc_switch_core::commands::proxy::update_circuit_breaker_config(&app_state, config).await {
                        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                None => Err("missing config".to_string()),
            }
        }
        "get_circuit_breaker_stats" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::proxy::get_circuit_breaker_stats(&app_state, &provider_id, &app_type).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "is_portable_mode" => match cc_switch_core::commands::misc::is_portable_mode() {
            Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
            Err(e) => Err(e.to_string()),
        },
        "get_init_error" => {
            match cc_switch_core::commands::misc::get_init_error_command() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_migration_result" => {
            match cc_switch_core::commands::misc::get_migration_result() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_skills_migration_result" => {
            match cc_switch_core::commands::misc::get_skills_migration_result() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "check_for_updates" => {
            match cc_switch_core::commands::misc::check_for_updates(&*platform).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "copy_text_to_clipboard" => {
            let text = req
                .args
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::misc::copy_text_to_clipboard(&*platform, text).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "open_external" => {
            let url = req
                .args
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::misc::open_external(&*platform, url).await {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "s3_test_connection" => match req.args.get("settings") {
            Some(settings) => {
                match serde_json::from_value::<cc_switch_core::settings::S3SyncSettings>(
                    settings.clone(),
                ) {
                    Ok(settings) => {
                        let preserve_empty_password = req
                            .args
                            .get("preserveEmptyPassword")
                            .and_then(|v| v.as_bool());
                        cc_switch_core::commands::s3_sync::s3_test_connection(
                            settings,
                            preserve_empty_password,
                        )
                        .await
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            None => Err("missing settings".to_string()),
        },
        "s3_sync_upload" => cc_switch_core::commands::s3_sync::s3_sync_upload(&app_state).await,
        "s3_sync_download" => cc_switch_core::commands::s3_sync::s3_sync_download(&app_state).await,
        "s3_sync_save_settings" => match req.args.get("settings") {
            Some(settings) => {
                match serde_json::from_value::<cc_switch_core::settings::S3SyncSettings>(
                    settings.clone(),
                ) {
                    Ok(settings) => {
                        let password_touched = req
                            .args
                            .get("passwordTouched")
                            .and_then(|v| v.as_bool());
                        cc_switch_core::commands::s3_sync::s3_sync_save_settings(
                            settings,
                            password_touched,
                        )
                        .await
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            None => Err("missing settings".to_string()),
        },
        "s3_sync_fetch_remote_info" => {
            cc_switch_core::commands::s3_sync::s3_sync_fetch_remote_info().await
        }
        "webdav_test_connection" => match req.args.get("settings") {
            Some(settings) => {
                match serde_json::from_value::<cc_switch_core::settings::WebDavSyncSettings>(
                    settings.clone(),
                ) {
                    Ok(settings) => {
                        let preserve_empty_password = req
                            .args
                            .get("preserveEmptyPassword")
                            .and_then(|v| v.as_bool());
                        cc_switch_core::commands::webdav_sync::webdav_test_connection(
                            settings,
                            preserve_empty_password,
                        )
                        .await
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            None => Err("missing settings".to_string()),
        },
        "webdav_sync_upload" => {
            cc_switch_core::commands::webdav_sync::webdav_sync_upload(&app_state).await
        }
        "webdav_sync_download" => {
            cc_switch_core::commands::webdav_sync::webdav_sync_download(&app_state).await
        }
        "webdav_sync_save_settings" => match req.args.get("settings") {
            Some(settings) => {
                match serde_json::from_value::<cc_switch_core::settings::WebDavSyncSettings>(
                    settings.clone(),
                ) {
                    Ok(settings) => {
                        let password_touched = req
                            .args
                            .get("passwordTouched")
                            .and_then(|v| v.as_bool());
                        cc_switch_core::commands::webdav_sync::webdav_sync_save_settings(
                            settings,
                            password_touched,
                        )
                        .await
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            None => Err("missing settings".to_string()),
        },
        "webdav_sync_fetch_remote_info" => {
            cc_switch_core::commands::webdav_sync::webdav_sync_fetch_remote_info().await
        }
        "get_claude_config_status" => {
            match cc_switch_core::commands::config::get_claude_config_status() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_claude_common_config_snippet" => {
            match cc_switch_core::commands::config::get_claude_common_config_snippet(&db) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_claude_common_config_snippet" => {
            let snippet = req
                .args
                .get("snippet")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::config::set_claude_common_config_snippet(&db, &snippet) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_common_config_snippet" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::config::get_common_config_snippet(&db, &app_type) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "update_toml_common_config_snippet" => {
            let config_toml = req
                .args
                .get("configToml")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let snippet_toml = req
                .args
                .get("snippetToml")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let enabled = req
                .args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match cc_switch_core::commands::config::update_toml_common_config_snippet(
                &config_toml,
                &snippet_toml,
                enabled,
            ) {
                Ok(v) => Ok(Value::String(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_common_config_snippet" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let snippet = req
                .args
                .get("snippet")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::config::set_common_config_snippet(
                &app_state,
                &app_type,
                &snippet,
            ) {
                Ok(()) => Ok(Value::Bool(true)),
                Err(e) => Err(e.to_string()),
            }
        }
        "extract_common_config_snippet" => {
            let app_type = req
                .args
                .get("appType")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let settings_config = req
                .args
                .get("settingsConfig")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            match cc_switch_core::commands::config::extract_common_config_snippet(
                &app_state,
                &app_type,
                settings_config.as_deref(),
            ) {
                Ok(v) => Ok(Value::String(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "create_db_backup" => {
            let db = db.clone();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::import_export::create_db_backup(&db)
            })
            .await
            {
                Ok(Ok(v)) => Ok(Value::String(v)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "list_db_backups" => {
            match tokio::task::spawn_blocking(|| {
                cc_switch_core::commands::import_export::list_db_backups()
            })
            .await
            {
                Ok(Ok(v)) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "restore_db_backup" => {
            let filename = req
                .args
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let db = db.clone();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::import_export::restore_db_backup(&db, &filename)
            })
            .await
            {
                Ok(Ok(v)) => Ok(Value::String(v)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "rename_db_backup" => {
            let old_filename = req
                .args
                .get("oldFilename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let new_name = req
                .args
                .get("newName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::import_export::rename_db_backup(&old_filename, &new_name)
            })
            .await
            {
                Ok(Ok(v)) => Ok(Value::String(v)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_db_backup" => {
            let filename = req
                .args
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::import_export::delete_db_backup(&filename)
            })
            .await
            {
                Ok(Ok(())) => Ok(Value::Bool(true)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "scan_local_proxies" => {
            let found = cc_switch_core::commands::global_proxy::scan_local_proxies().await;
            Ok(serde_json::to_value(found).unwrap_or(Value::Null))
        }
        "get_mcp_config" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            match cc_switch_core::commands::mcp::get_mcp_config(&app_state, &app) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "upsert_mcp_server_in_config" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let spec = req.args.get("spec").cloned().unwrap_or(Value::Null);
            let sync_other_side = req.args.get("syncOtherSide").and_then(|v| v.as_bool());
            match cc_switch_core::commands::mcp::upsert_mcp_server_in_config(
                &app_state,
                &app,
                &id,
                spec,
                sync_other_side,
            ) {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_mcp_server_in_config" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::mcp::delete_mcp_server_in_config(
                &app_state,
                &app,
                &id,
            ) {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_mcp_enabled" => {
            let app = req
                .args
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude")
                .to_string();
            let id = req
                .args
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let enabled = req
                .args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match cc_switch_core::commands::mcp::set_mcp_enabled(
                &app_state,
                &app,
                &id,
                enabled,
            ) {
                Ok(v) => Ok(Value::Bool(v)),
                Err(e) => Err(e.to_string()),
            }
        }
        "import_mcp_from_apps" => {
            match cc_switch_core::commands::mcp::import_mcp_from_apps(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- Hermes 配置命令（8 个 A 类）-----
        "import_hermes_providers_from_live" => {
            match cc_switch_core::commands::hermes::import_hermes_providers_from_live(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_hermes_live_provider_ids" => {
            match cc_switch_core::commands::hermes::get_hermes_live_provider_ids() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_hermes_live_provider" => {
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::hermes::get_hermes_live_provider(&provider_id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_hermes_model_config" => {
            match cc_switch_core::commands::hermes::get_hermes_model_config() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_hermes_memory" => {
            let kind_value = req.args.get("kind").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::hermes_config::MemoryKind>(kind_value) {
                Ok(kind) => match cc_switch_core::commands::hermes::get_hermes_memory(kind) {
                    Ok(v) => Ok(Value::String(v)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "set_hermes_memory" => {
            let kind_value = req.args.get("kind").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::hermes_config::MemoryKind>(kind_value) {
                Ok(kind) => {
                    let content = req
                        .args
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    match cc_switch_core::commands::hermes::set_hermes_memory(kind, &content) {
                        Ok(()) => Ok(Value::Bool(true)),
                        Err(e) => Err(e.to_string()),
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        }
        "get_hermes_memory_limits" => {
            match cc_switch_core::commands::hermes::get_hermes_memory_limits() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_hermes_memory_enabled" => {
            let kind_value = req.args.get("kind").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::hermes_config::MemoryKind>(kind_value) {
                Ok(kind) => {
                    let enabled = req
                        .args
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    match cc_switch_core::commands::hermes::set_hermes_memory_enabled(kind, enabled) {
                        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                        Err(e) => Err(e.to_string()),
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- OpenClaw 配置命令（14 个 A 类）-----
        "import_openclaw_providers_from_live" => {
            match cc_switch_core::commands::openclaw::import_openclaw_providers_from_live(&app_state) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_openclaw_live_provider_ids" => {
            match cc_switch_core::commands::openclaw::get_openclaw_live_provider_ids() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_openclaw_live_provider" => {
            let provider_id = req
                .args
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match cc_switch_core::commands::openclaw::get_openclaw_live_provider(&provider_id) {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "scan_openclaw_config_health" => {
            match cc_switch_core::commands::openclaw::scan_openclaw_config_health() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "get_openclaw_default_model" => {
            match cc_switch_core::commands::openclaw::get_openclaw_default_model() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_openclaw_default_model" => {
            let model_value = req.args.get("model").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::openclaw_config::OpenClawDefaultModel>(model_value) {
                Ok(model) => match cc_switch_core::commands::openclaw::set_openclaw_default_model(model) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_openclaw_model_catalog" => {
            match cc_switch_core::commands::openclaw::get_openclaw_model_catalog() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_openclaw_model_catalog" => {
            let catalog_value = req.args.get("catalog").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<std::collections::HashMap<
                String,
                cc_switch_core::openclaw_config::OpenClawModelCatalogEntry,
            >>(catalog_value) {
                Ok(catalog) => match cc_switch_core::commands::openclaw::set_openclaw_model_catalog(catalog) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_openclaw_agents_defaults" => {
            match cc_switch_core::commands::openclaw::get_openclaw_agents_defaults() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_openclaw_agents_defaults" => {
            let defaults_value = req.args.get("defaults").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::openclaw_config::OpenClawAgentsDefaults>(defaults_value) {
                Ok(defaults) => match cc_switch_core::commands::openclaw::set_openclaw_agents_defaults(defaults) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_openclaw_env" => {
            match cc_switch_core::commands::openclaw::get_openclaw_env() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_openclaw_env" => {
            let env_value = req.args.get("env").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::openclaw_config::OpenClawEnvConfig>(env_value) {
                Ok(env) => match cc_switch_core::commands::openclaw::set_openclaw_env(env) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        "get_openclaw_tools" => {
            match cc_switch_core::commands::openclaw::get_openclaw_tools() {
                Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Err(e) => Err(e.to_string()),
            }
        }
        "set_openclaw_tools" => {
            let tools_value = req.args.get("tools").cloned().unwrap_or(Value::Null);
            match serde_json::from_value::<cc_switch_core::openclaw_config::OpenClawToolsConfig>(tools_value) {
                Ok(tools) => match cc_switch_core::commands::openclaw::set_openclaw_tools(tools) {
                    Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        // ----- OpenClaw workspace 文件命令（7 个 A 类）-----
        "list_daily_memory_files" => {
            match tokio::task::spawn_blocking(|| {
                cc_switch_core::commands::workspace::list_daily_memory_files()
            })
            .await
            {
                Ok(Ok(v)) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "read_daily_memory_file" => {
            let filename = req
                .args
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::workspace::read_daily_memory_file(&filename)
            })
            .await
            {
                Ok(Ok(v)) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "write_daily_memory_file" => {
            let filename = req
                .args
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let content = req
                .args
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::workspace::write_daily_memory_file(&filename, &content)
            })
            .await
            {
                Ok(Ok(())) => Ok(Value::Bool(true)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "search_daily_memory_files" => {
            let query = req
                .args
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::workspace::search_daily_memory_files(&query)
            })
            .await
            {
                Ok(Ok(v)) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "delete_daily_memory_file" => {
            let filename = req
                .args
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::workspace::delete_daily_memory_file(&filename)
            })
            .await
            {
                Ok(Ok(())) => Ok(Value::Bool(true)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "read_workspace_file" => {
            let filename = req
                .args
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::workspace::read_workspace_file(&filename)
            })
            .await
            {
                Ok(Ok(v)) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        "write_workspace_file" => {
            let filename = req
                .args
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let content = req
                .args
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            match tokio::task::spawn_blocking(move || {
                cc_switch_core::commands::workspace::write_workspace_file(&filename, &content)
            })
            .await
            {
                Ok(Ok(())) => Ok(Value::Bool(true)),
                Ok(Err(e)) => Err(e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
        _ => Err(format!("unknown command: {}", req.cmd)),
    };

    match result {
        Ok(data) => {
            let resp = InvokeResponse {
                success: true,
                data: Some(data),
                error: None,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(error) => {
            let resp = InvokeResponse {
                success: false,
                data: None,
                error: Some(error),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
    }
}

async fn version_handler(Extension(platform): Extension<Arc<dyn Platform>>) -> Response {
    Json(serde_json::json!({
        "version": platform.app_version(),
    }))
    .into_response()
}

async fn info_handler(
    Extension(platform): Extension<Arc<dyn Platform>>,
    Extension(app_state): Extension<AppState>,
) -> Response {
    Json(serde_json::json!({
        "version": platform.app_version(),
        "appConfigDir": platform.app_config_dir(),
        "homeDir": dirs::home_dir(),
        "providersEmpty": app_state.db.is_providers_empty().unwrap_or(true),
    }))
    .into_response()
}

// ============================================================================
// 文件上传/下载端点（P4-A：用于 Web 模式下的文件对话框 shim）
// ============================================================================

/// 上传临时文件，返回服务器路径供后续 invoke 命令使用。
///
/// 前端 shim `plugin-dialog.ts` 在 Web 模式下用 `<input type="file">` 选文件后，
/// 自动 POST 到此端点；服务器保存到临时目录，返回路径字符串。
/// 后续 invoke 命令（如 `import_config_from_file`）用此路径读取文件。
async fn upload_handler(mut multipart: Multipart) -> Response {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "upload.bin".to_string());
        let data = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("读取上传数据失败: {e}"))
                    .into_response();
            }
        };
        let temp_dir = std::env::temp_dir().join("cc-switch-web-uploads");
        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("创建上传目录失败: {e}"))
                .into_response();
        }
        let id = uuid::Uuid::new_v4();
        let suffix = std::path::Path::new(&file_name)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let saved_name = format!("upload-{id}{suffix}");
        let saved_path = temp_dir.join(&saved_name);
        if let Err(e) = std::fs::write(&saved_path, &data) {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("保存上传文件失败: {e}"))
                .into_response();
        }
        log::info!("[upload] {} -> {}", file_name, saved_path.display());
        return Json(serde_json::json!({
            "path": saved_path.to_string_lossy(),
            "originalName": file_name,
            "size": data.len(),
        }))
        .into_response();
    }
    (StatusCode::BAD_REQUEST, "未收到上传文件").into_response()
}

/// 下载服务器端文件（P4-A：save_file_dialog 的 Web 实现）。
///
/// 前端 shim `plugin-dialog.ts` 在 Web 模式下用 `<a download>` 触发浏览器下载，
/// 但需要先从此端点拿数据。此端点返回原始字节，Content-Disposition 触发下载。
async fn download_handler(Path(filename): Path<String>) -> Response {
    let temp_dir = std::env::temp_dir().join("cc-switch-web-uploads");
    let path = temp_dir.join(&filename);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "文件不存在").into_response();
    }
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("读取失败: {e}")).into_response(),
    };
    let display_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(filename);
    (
        StatusCode::OK,
        [
            ("Content-Type", "application/octet-stream".to_string()),
            (
                "Content-Disposition",
                format!("attachment; filename=\"{display_name}\""),
            ),
        ],
        data,
    )
        .into_response()
}

// ============================================================================
// 服务重启端点（P4-A：restart_app 的 Web 实现）
// ============================================================================

/// 触发 axum 优雅关闭，依赖 systemd `Restart=on-failure`/`Restart=always` 自动重启。
///
/// 调用方（前端）应在收到 200 响应后断开 EventSource，等待服务恢复后重连。
/// systemd unit 配置示例：
/// ```ini
/// [Service]
/// Type=simple
/// Restart=on-failure
/// RestartSec=2s
/// ```
async fn restart_handler() -> Response {
    log::info!("[restart] received restart request, triggering graceful shutdown");
    // 用全局 shutdown channel 通知 main 退出 axum::serve
    // main.rs 持有 receiver，收到信号后 axum 走 graceful shutdown
    if let Some(sender) = SHUTDOWN_SENDER.get() {
        let _ = sender.send(());
    }
    Json(serde_json::json!({
        "success": true,
        "message": "Restart triggered. The service will be back in a few seconds.",
    }))
    .into_response()
}

/// 全局 shutdown sender，由 main.rs 在启动时注入。
static SHUTDOWN_SENDER: std::sync::OnceLock<tokio::sync::mpsc::Sender<()>> =
    std::sync::OnceLock::new();

/// 在 main.rs 启动时调用，注入 shutdown 信号 sender。
pub fn set_shutdown_sender(sender: tokio::sync::mpsc::Sender<()>) {
    let _ = SHUTDOWN_SENDER.set(sender);
}

// ============================================================================
// 文件对话框命令兜底（P4-A：由前端 shim 处理，后端只兜底未知情况）
// ============================================================================
