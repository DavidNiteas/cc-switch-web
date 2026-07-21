//! 应用存储相关功能（POC 阶段）。
//!
//! 桌面版通过 tauri_plugin_store 保存覆盖路径，core 中仅维护内存缓存。
//! 实际持久化逻辑由调用方（tauri/web）在初始化时注入。

use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

static APP_CONFIG_DIR_OVERRIDE: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();

fn override_cache() -> &'static RwLock<Option<PathBuf>> {
    APP_CONFIG_DIR_OVERRIDE.get_or_init(|| RwLock::new(None))
}

/// 设置 app_config_dir 覆盖路径
pub fn set_app_config_dir_override(value: Option<PathBuf>) {
    if let Ok(mut guard) = override_cache().write() {
        *guard = value;
    }
}

/// 获取缓存中的 app_config_dir 覆盖路径
pub fn get_app_config_dir_override() -> Option<PathBuf> {
    override_cache().read().ok()?.clone()
}
