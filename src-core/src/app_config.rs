use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use crate::error::AppError;
use crate::prompt::Prompt;
use crate::provider::ProviderManager;
use crate::services::skill::SkillStore;

/// 应用类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    Claude,
    #[serde(
        rename = "claude-desktop",
        alias = "claude_desktop",
        alias = "claudeDesktop"
    )]
    ClaudeDesktop,
    Codex,
    Gemini,
    GrokBuild,
    OpenCode,
    OpenClaw,
    Hermes,
}

impl AppType {
    pub fn as_str(&self) -> &str {
        match self {
            AppType::Claude => "claude",
            AppType::ClaudeDesktop => "claude-desktop",
            AppType::Codex => "codex",
            AppType::Gemini => "gemini",
            AppType::GrokBuild => "grokbuild",
            AppType::OpenCode => "opencode",
            AppType::OpenClaw => "openclaw",
            AppType::Hermes => "hermes",
        }
    }

    /// Check if this app uses additive mode
    pub fn is_additive_mode(&self) -> bool {
        matches!(
            self,
            AppType::OpenCode | AppType::OpenClaw | AppType::Hermes
        )
    }

    /// Return an iterator over all app types
    pub fn all() -> impl Iterator<Item = AppType> {
        [
            AppType::Claude,
            AppType::ClaudeDesktop,
            AppType::Codex,
            AppType::Gemini,
            AppType::GrokBuild,
            AppType::OpenCode,
            AppType::OpenClaw,
            AppType::Hermes,
        ]
        .into_iter()
    }
}

impl FromStr for AppType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "claude" => Ok(AppType::Claude),
            "claude-desktop" | "claude_desktop" | "claudedesktop" => Ok(AppType::ClaudeDesktop),
            "codex" => Ok(AppType::Codex),
            "gemini" => Ok(AppType::Gemini),
            "grokbuild" | "grok-build" | "grok_build" | "grok" => Ok(AppType::GrokBuild),
            "opencode" => Ok(AppType::OpenCode),
            "openclaw" => Ok(AppType::OpenClaw),
            "hermes" => Ok(AppType::Hermes),
            other => Err(AppError::localized(
                "unsupported_app",
                format!("不支持的应用标识: '{other}'。可选值: claude, claude-desktop, codex, gemini, grokbuild, opencode, openclaw, hermes。"),
                format!("Unsupported app id: '{other}'. Allowed: claude, claude-desktop, codex, gemini, grokbuild, opencode, openclaw, hermes."),
            )),
        }
    }
}

/// MCP 服务器应用状态（标记应用到哪些客户端）
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct McpApps {
    #[serde(default)]
    pub claude: bool,
    #[serde(default)]
    pub codex: bool,
    #[serde(default)]
    pub gemini: bool,
    #[serde(default)]
    pub grokbuild: bool,
    #[serde(default)]
    pub opencode: bool,
    #[serde(default)]
    pub hermes: bool,
}

impl McpApps {
    /// 检查指定应用是否启用
    pub fn is_enabled_for(&self, app: &AppType) -> bool {
        match app {
            AppType::Claude => self.claude,
            AppType::Codex => self.codex,
            AppType::Gemini => self.gemini,
            AppType::GrokBuild => self.grokbuild,
            AppType::OpenCode => self.opencode,
            AppType::OpenClaw => false, // OpenClaw doesn't support MCP
            AppType::Hermes => self.hermes,
            AppType::ClaudeDesktop => false,
        }
    }

    /// 设置指定应用的启用状态
    pub fn set_enabled_for(&mut self, app: &AppType, enabled: bool) {
        match app {
            AppType::Claude => self.claude = enabled,
            AppType::Codex => self.codex = enabled,
            AppType::Gemini => self.gemini = enabled,
            AppType::GrokBuild => self.grokbuild = enabled,
            AppType::OpenCode => self.opencode = enabled,
            AppType::OpenClaw => {} // OpenClaw doesn't support MCP, ignore
            AppType::Hermes => self.hermes = enabled,
            AppType::ClaudeDesktop => {} // Claude Desktop 3P provider config doesn't support MCP here
        }
    }

    /// 获取所有启用的应用列表
    pub fn enabled_apps(&self) -> Vec<AppType> {
        let mut apps = Vec::new();
        if self.claude {
            apps.push(AppType::Claude);
        }
        if self.codex {
            apps.push(AppType::Codex);
        }
        if self.gemini {
            apps.push(AppType::Gemini);
        }
        if self.grokbuild {
            apps.push(AppType::GrokBuild);
        }
        if self.opencode {
            apps.push(AppType::OpenCode);
        }
        if self.hermes {
            apps.push(AppType::Hermes);
        }
        apps
    }

    /// 检查是否所有应用都未启用
    pub fn is_empty(&self) -> bool {
        !self.claude
            && !self.codex
            && !self.gemini
            && !self.grokbuild
            && !self.opencode
            && !self.hermes
    }
}

/// Skill 应用启用状态（标记 Skill 应用到哪些客户端）
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SkillApps {
    #[serde(default)]
    pub claude: bool,
    #[serde(default)]
    pub codex: bool,
    #[serde(default)]
    pub gemini: bool,
    #[serde(default)]
    pub grokbuild: bool,
    #[serde(default)]
    pub opencode: bool,
    #[serde(default)]
    pub hermes: bool,
}

impl SkillApps {
    /// 检查指定应用是否启用
    pub fn is_enabled_for(&self, app: &AppType) -> bool {
        match app {
            AppType::Claude => self.claude,
            AppType::Codex => self.codex,
            AppType::Gemini => self.gemini,
            AppType::GrokBuild => self.grokbuild,
            AppType::OpenCode => self.opencode,
            AppType::Hermes => self.hermes,
            AppType::OpenClaw => false, // OpenClaw doesn't support Skills
            AppType::ClaudeDesktop => false,
        }
    }

    /// 设置指定应用的启用状态
    pub fn set_enabled_for(&mut self, app: &AppType, enabled: bool) {
        match app {
            AppType::Claude => self.claude = enabled,
            AppType::Codex => self.codex = enabled,
            AppType::Gemini => self.gemini = enabled,
            AppType::GrokBuild => self.grokbuild = enabled,
            AppType::OpenCode => self.opencode = enabled,
            AppType::Hermes => self.hermes = enabled,
            AppType::OpenClaw => {} // OpenClaw doesn't support Skills, ignore
            AppType::ClaudeDesktop => {} // Claude Desktop 3P profiles don't use CC Switch skill sync
        }
    }

    /// 获取所有启用的应用列表
    pub fn enabled_apps(&self) -> Vec<AppType> {
        let mut apps = Vec::new();
        if self.claude {
            apps.push(AppType::Claude);
        }
        if self.codex {
            apps.push(AppType::Codex);
        }
        if self.gemini {
            apps.push(AppType::Gemini);
        }
        if self.grokbuild {
            apps.push(AppType::GrokBuild);
        }
        if self.opencode {
            apps.push(AppType::OpenCode);
        }
        if self.hermes {
            apps.push(AppType::Hermes);
        }
        apps
    }

    /// 检查是否所有应用都未启用
    pub fn is_empty(&self) -> bool {
        !self.claude
            && !self.codex
            && !self.gemini
            && !self.grokbuild
            && !self.opencode
            && !self.hermes
    }

    /// 仅启用指定应用（其他应用设为禁用）
    pub fn only(app: &AppType) -> Self {
        let mut apps = Self::default();
        apps.set_enabled_for(app, true);
        apps
    }

    /// 从来源标签列表构建启用状态
    ///
    /// 标签与 AppType::as_str() 一致时启用对应应用，
    /// 其他标签（如 "agents", "cc-switch"）忽略。
    pub fn from_labels(labels: &[String]) -> Self {
        let mut apps = Self::default();
        for label in labels {
            if let Ok(app) = label.parse::<AppType>() {
                apps.set_enabled_for(&app, true);
            }
        }
        apps
    }
}

/// 已安装的 Skill（v3.10.0+ 统一结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    /// 唯一标识符（格式："owner/repo:directory" 或 "local:directory"）
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 安装目录名（在 SSOT 目录中的子目录名）
    pub directory: String,
    /// 仓库所有者（GitHub 用户/组织）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_owner: Option<String>,
    /// 仓库名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    /// 仓库分支
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_branch: Option<String>,
    /// README URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme_url: Option<String>,
    /// 应用启用状态
    pub apps: SkillApps,
    /// 安装时间（Unix 时间戳）
    pub installed_at: i64,
    /// 内容哈希（SHA-256，用于更新检测）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// 最近更新时间（Unix 时间戳，0 = 从未更新）
    #[serde(default)]
    pub updated_at: i64,
}

/// 未管理的 Skill（在应用目录中发现但未被 CC Switch 管理）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnmanagedSkill {
    /// 目录名
    pub directory: String,
    /// 显示名称（从 SKILL.md 解析）
    pub name: String,
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 在哪些应用目录中发现（如 ["claude", "codex"]）
    pub found_in: Vec<String>,
    /// 发现路径（首个匹配的完整路径）
    pub path: String,
}

/// MCP 服务器定义（v3.7.0 统一结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub server: serde_json::Value,
    pub apps: McpApps,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// MCP 配置：单客户端维度（v3.6.x 及以前，保留用于向后兼容）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// 以 id 为键的服务器定义（宽松 JSON 对象，包含 enabled/source 等 UI 辅助字段）
    #[serde(default)]
    pub servers: HashMap<String, serde_json::Value>,
}

impl McpConfig {
    /// 检查配置是否为空
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }
}

/// MCP 根配置（v3.7.0 新旧结构并存）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRoot {
    /// 统一的 MCP 服务器存储（v3.7.0+）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<HashMap<String, McpServer>>,

    /// 旧的分应用存储（v3.6.x 及以前，保留用于迁移）
    #[serde(default, skip_serializing_if = "McpConfig::is_empty")]
    pub claude: McpConfig,
    #[serde(
        rename = "claude-desktop",
        alias = "claudeDesktop",
        alias = "claude_desktop",
        default,
        skip_serializing_if = "McpConfig::is_empty"
    )]
    pub claude_desktop: McpConfig,
    #[serde(default, skip_serializing_if = "McpConfig::is_empty")]
    pub codex: McpConfig,
    #[serde(default, skip_serializing_if = "McpConfig::is_empty")]
    pub gemini: McpConfig,
    #[serde(default, skip_serializing_if = "McpConfig::is_empty")]
    pub grokbuild: McpConfig,
    /// OpenCode MCP 配置（v4.0.0+，实际使用 opencode.json）
    #[serde(default, skip_serializing_if = "McpConfig::is_empty")]
    pub opencode: McpConfig,
    /// OpenClaw MCP 配置（v4.1.0+，实际使用 openclaw.json）
    #[serde(default, skip_serializing_if = "McpConfig::is_empty")]
    pub openclaw: McpConfig,
    /// Hermes MCP 配置（实际使用 config.yaml）
    #[serde(default, skip_serializing_if = "McpConfig::is_empty")]
    pub hermes: McpConfig,
}

impl Default for McpRoot {
    fn default() -> Self {
        Self {
            // v3.7.0+ 默认使用新的统一结构（空 HashMap）
            servers: Some(HashMap::new()),
            // 旧结构保持空，仅用于反序列化旧配置时的迁移
            claude: McpConfig::default(),
            claude_desktop: McpConfig::default(),
            codex: McpConfig::default(),
            gemini: McpConfig::default(),
            grokbuild: McpConfig::default(),
            opencode: McpConfig::default(),
            openclaw: McpConfig::default(),
            hermes: McpConfig::default(),
        }
    }
}

/// Prompt 配置：单客户端维度
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptConfig {
    #[serde(default)]
    pub prompts: HashMap<String, Prompt>,
}

/// Prompt 根：按客户端分开维护
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptRoot {
    #[serde(default)]
    pub claude: PromptConfig,
    #[serde(
        rename = "claude-desktop",
        alias = "claudeDesktop",
        alias = "claude_desktop",
        default
    )]
    pub claude_desktop: PromptConfig,
    #[serde(default)]
    pub codex: PromptConfig,
    #[serde(default)]
    pub gemini: PromptConfig,
    #[serde(default)]
    pub grokbuild: PromptConfig,
    #[serde(default)]
    pub opencode: PromptConfig,
    #[serde(default)]
    pub openclaw: PromptConfig,
    #[serde(default)]
    pub hermes: PromptConfig,
}

/// 通用配置片段（按应用分治）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonConfigSnippets {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opencode: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openclaw: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hermes: Option<String>,
}

impl CommonConfigSnippets {
    /// 获取指定应用的通用配置片段
    pub fn get(&self, app: &AppType) -> Option<&String> {
        match app {
            AppType::Claude => self.claude.as_ref(),
            AppType::ClaudeDesktop => None,
            AppType::Codex => self.codex.as_ref(),
            AppType::Gemini => self.gemini.as_ref(),
            AppType::GrokBuild => None,
            AppType::OpenCode => self.opencode.as_ref(),
            AppType::OpenClaw => self.openclaw.as_ref(),
            AppType::Hermes => self.hermes.as_ref(),
        }
    }

    /// 设置指定应用的通用配置片段
    pub fn set(&mut self, app: &AppType, snippet: Option<String>) {
        match app {
            AppType::Claude => self.claude = snippet,
            AppType::ClaudeDesktop => {}
            AppType::Codex => self.codex = snippet,
            AppType::Gemini => self.gemini = snippet,
            AppType::GrokBuild => {}
            AppType::OpenCode => self.opencode = snippet,
            AppType::OpenClaw => self.openclaw = snippet,
            AppType::Hermes => self.hermes = snippet,
        }
    }
}

/// 多应用配置结构（向后兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAppConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    /// 应用管理器（claude/codex）
    #[serde(flatten)]
    pub apps: HashMap<String, ProviderManager>,
    /// MCP 配置（按客户端分治）
    #[serde(default)]
    pub mcp: McpRoot,
    /// Prompt 配置（按客户端分治）
    #[serde(default)]
    pub prompts: PromptRoot,
    /// Claude Skills 配置
    #[serde(default)]
    pub skills: SkillStore,
    /// 通用配置片段（按应用分治）
    #[serde(default)]
    pub common_config_snippets: CommonConfigSnippets,
    /// Claude 通用配置片段（旧字段，用于向后兼容迁移）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_common_config_snippet: Option<String>,
}

fn default_version() -> u32 {
    2
}

impl Default for MultiAppConfig {
    fn default() -> Self {
        let mut apps = HashMap::new();
        apps.insert("claude".to_string(), ProviderManager::default());
        apps.insert("claude-desktop".to_string(), ProviderManager::default());
        apps.insert("codex".to_string(), ProviderManager::default());
        apps.insert("gemini".to_string(), ProviderManager::default());
        apps.insert("grokbuild".to_string(), ProviderManager::default());
        apps.insert("opencode".to_string(), ProviderManager::default());
        apps.insert("openclaw".to_string(), ProviderManager::default());
        apps.insert("hermes".to_string(), ProviderManager::default());

        Self {
            version: 2,
            apps,
            mcp: McpRoot::default(),
            prompts: PromptRoot::default(),
            skills: SkillStore::default(),
            common_config_snippets: CommonConfigSnippets::default(),
            claude_common_config_snippet: None,
        }
    }
}

impl MultiAppConfig {
    /// 获取指定应用的管理器
    pub fn get_manager(&self, app: &AppType) -> Option<&ProviderManager> {
        self.apps.get(app.as_str())
    }

    /// 获取指定应用的管理器（可变引用）
    pub fn get_manager_mut(&mut self, app: &AppType) -> Option<&mut ProviderManager> {
        self.apps.get_mut(app.as_str())
    }

    /// 确保应用存在
    pub fn ensure_app(&mut self, app: &AppType) {
        if !self.apps.contains_key(app.as_str()) {
            self.apps
                .insert(app.as_str().to_string(), ProviderManager::default());
        }
    }

    /// 获取指定客户端的 MCP 配置（不可变引用）
    pub fn mcp_for(&self, app: &AppType) -> &McpConfig {
        match app {
            AppType::Claude => &self.mcp.claude,
            AppType::ClaudeDesktop => &self.mcp.claude_desktop,
            AppType::Codex => &self.mcp.codex,
            AppType::Gemini => &self.mcp.gemini,
            AppType::GrokBuild => &self.mcp.grokbuild,
            AppType::OpenCode => &self.mcp.opencode,
            AppType::OpenClaw => &self.mcp.openclaw,
            AppType::Hermes => &self.mcp.hermes,
        }
    }

    /// 获取指定客户端的 MCP 配置（可变引用）
    pub fn mcp_for_mut(&mut self, app: &AppType) -> &mut McpConfig {
        match app {
            AppType::Claude => &mut self.mcp.claude,
            AppType::ClaudeDesktop => &mut self.mcp.claude_desktop,
            AppType::Codex => &mut self.mcp.codex,
            AppType::Gemini => &mut self.mcp.gemini,
            AppType::GrokBuild => &mut self.mcp.grokbuild,
            AppType::OpenCode => &mut self.mcp.opencode,
            AppType::OpenClaw => &mut self.mcp.openclaw,
            AppType::Hermes => &mut self.mcp.hermes,
        }
    }
}
