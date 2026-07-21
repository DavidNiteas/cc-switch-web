use crate::proxy::providers::codex_oauth_auth::CodexOAuthManager;
use crate::proxy::providers::copilot_auth::CopilotAuthManager;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProxyAuthState {
    pub copilot: Arc<RwLock<CopilotAuthManager>>,
    pub codex_oauth: Arc<RwLock<CodexOAuthManager>>,
}

impl ProxyAuthState {
    pub fn new() -> Self {
        let data_dir = crate::config::get_app_config_dir();
        Self {
            copilot: Arc::new(RwLock::new(CopilotAuthManager::new(data_dir.clone()))),
            codex_oauth: Arc::new(RwLock::new(CodexOAuthManager::new(data_dir))),
        }
    }
}

impl Default for ProxyAuthState {
    fn default() -> Self {
        Self::new()
    }
}
