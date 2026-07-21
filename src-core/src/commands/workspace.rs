//! OpenClaw workspace 文件读写命令（A 类，纯文件系统操作）。
//!
//! 对应 tauri 侧 `commands/workspace.rs` 中的 7 个 A 类命令。
//! `open_workspace_directory` 是 D 类（使用系统文件管理器），保留在 tauri 外壳。

use regex::Regex;
use std::sync::LazyLock;

use crate::config::write_text_file;
use crate::error::AppError;
use crate::openclaw_config::get_openclaw_dir;

/// 允许操作的 workspace 文件名白名单。
const ALLOWED_FILES: &[&str] = &[
    "AGENTS.md",
    "SOUL.md",
    "USER.md",
    "IDENTITY.md",
    "TOOLS.md",
    "MEMORY.md",
    "HEARTBEAT.md",
    "BOOTSTRAP.md",
    "BOOT.md",
];

fn validate_filename(filename: &str) -> Result<(), AppError> {
    if !ALLOWED_FILES.contains(&filename) {
        return Err(AppError::Message(format!(
            "Invalid workspace filename: {filename}. Allowed: {}",
            ALLOWED_FILES.join(", ")
        )));
    }
    Ok(())
}

static DAILY_MEMORY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}\.md$").unwrap());

fn validate_daily_memory_filename(filename: &str) -> Result<(), AppError> {
    if !DAILY_MEMORY_RE.is_match(filename) {
        return Err(AppError::Message(format!(
            "Invalid daily memory filename: {filename}. Expected: YYYY-MM-DD.md"
        )));
    }
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyMemoryFileInfo {
    pub filename: String,
    pub date: String,
    pub size_bytes: u64,
    pub modified_at: u64,
    pub preview: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyMemorySearchResult {
    pub filename: String,
    pub date: String,
    pub size_bytes: u64,
    pub modified_at: u64,
    pub snippet: String,
    pub match_count: usize,
}

/// 找到 `<= i` 的最大 UTF-8 字符边界索引。
fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// 找到 `>= i` 的最小 UTF-8 字符边界索引。
fn ceil_char_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn memory_dir() -> std::path::PathBuf {
    get_openclaw_dir().join("workspace").join("memory")
}

/// 列出 `workspace/memory/` 下所有日记文件，按文件名降序（最新日期在前）。
pub fn list_daily_memory_files() -> Result<Vec<DailyMemoryFileInfo>, AppError> {
    let memory_dir = memory_dir();

    if !memory_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files: Vec<DailyMemoryFileInfo> = Vec::new();
    let entries = std::fs::read_dir(&memory_dir)
        .map_err(|e| AppError::Message(format!("Failed to read memory directory: {e}")))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) if m.is_file() => m,
            _ => continue,
        };

        let date = name.trim_end_matches(".md").to_string();
        let size_bytes = meta.len();
        let modified_at = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let preview = std::fs::read_to_string(entry.path())
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect::<String>();

        files.push(DailyMemoryFileInfo {
            filename: name,
            date,
            size_bytes,
            modified_at,
            preview,
        });
    }

    files.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(files)
}

/// 读取指定日记文件内容，不存在时返回 None。
pub fn read_daily_memory_file(filename: &str) -> Result<Option<String>, AppError> {
    validate_daily_memory_filename(filename)?;

    let path = memory_dir().join(filename);
    if !path.exists() {
        return Ok(None);
    }

    std::fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| AppError::Message(format!("Failed to read daily memory file {filename}: {e}")))
}

/// 原子写入指定日记文件。
pub fn write_daily_memory_file(filename: &str, content: &str) -> Result<(), AppError> {
    validate_daily_memory_filename(filename)?;

    let memory_dir = memory_dir();
    std::fs::create_dir_all(&memory_dir)
        .map_err(|e| AppError::Message(format!("Failed to create memory directory: {e}")))?;

    let path = memory_dir.join(filename);
    write_text_file(&path, content)
        .map_err(|e| AppError::Message(format!("Failed to write daily memory file {filename}: {e}")))
}

/// 跨日记文件全文搜索（大小写不敏感），返回匹配结果。
pub fn search_daily_memory_files(query: &str) -> Result<Vec<DailyMemorySearchResult>, AppError> {
    let memory_dir = memory_dir();

    if !memory_dir.exists() || query.is_empty() {
        return Ok(Vec::new());
    }

    let query_lower = query.to_lowercase();
    let mut results: Vec<DailyMemorySearchResult> = Vec::new();

    let entries = std::fs::read_dir(&memory_dir)
        .map_err(|e| AppError::Message(format!("Failed to read memory directory: {e}")))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) if m.is_file() => m,
            _ => continue,
        };

        let date = name.trim_end_matches(".md").to_string();
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let content_lower = content.to_lowercase();

        let content_matches: Vec<usize> = content_lower
            .match_indices(&query_lower)
            .map(|(i, _)| i)
            .collect();
        let date_matches = date.to_lowercase().contains(&query_lower);

        if content_matches.is_empty() && !date_matches {
            continue;
        }

        let snippet = if let Some(&first_pos) = content_matches.first() {
            let start = if first_pos > 50 {
                floor_char_boundary(&content, first_pos - 50)
            } else {
                0
            };
            let end = ceil_char_boundary(&content, (first_pos + 70).min(content.len()));
            let mut s = String::new();
            if start > 0 {
                s.push_str("...");
            }
            s.push_str(&content[start..end]);
            if end < content.len() {
                s.push_str("...");
            }
            s
        } else {
            let end = ceil_char_boundary(&content, 120.min(content.len()));
            let mut s = content[..end].to_string();
            if end < content.len() {
                s.push_str("...");
            }
            s
        };

        let size_bytes = meta.len();
        let modified_at = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        results.push(DailyMemorySearchResult {
            filename: name,
            date,
            size_bytes,
            modified_at,
            snippet,
            match_count: content_matches.len(),
        });
    }

    results.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(results)
}

/// 删除指定日记文件（幂等）。
pub fn delete_daily_memory_file(filename: &str) -> Result<(), AppError> {
    validate_daily_memory_filename(filename)?;

    let path = memory_dir().join(filename);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| {
            AppError::Message(format!("Failed to delete daily memory file {filename}: {e}"))
        })?;
    }
    Ok(())
}

/// 读取 OpenClaw workspace 文件内容，不存在时返回 None。
pub fn read_workspace_file(filename: &str) -> Result<Option<String>, AppError> {
    validate_filename(filename)?;

    let path = get_openclaw_dir().join("workspace").join(filename);
    if !path.exists() {
        return Ok(None);
    }

    std::fs::read_to_string(&path).map(Some).map_err(|e| {
        AppError::Message(format!("Failed to read workspace file {filename}: {e}"))
    })
}

/// 原子写入 OpenClaw workspace 文件，自动创建 workspace 目录。
pub fn write_workspace_file(filename: &str, content: &str) -> Result<(), AppError> {
    validate_filename(filename)?;

    let workspace_dir = get_openclaw_dir().join("workspace");
    std::fs::create_dir_all(&workspace_dir)
        .map_err(|e| AppError::Message(format!("Failed to create workspace directory: {e}")))?;

    let path = workspace_dir.join(filename);
    write_text_file(&path, content)
        .map_err(|e| AppError::Message(format!("Failed to write workspace file {filename}: {e}")))
}
