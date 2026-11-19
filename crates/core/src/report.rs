use std::path::Path;

use serde::Serialize;

use crate::fsutil::display_relative;

/// 本次任务的操作类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Preview,
    Replace,
    Restore,
}

/// 单个文件的处理结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryStatus {
    Previewed,
    Written,
    Restored,
    Failed,
}

/// 报告中的一行记录。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportEntry {
    /// 绝对路径。
    pub path: String,
    /// 相对根目录的路径，供界面展示。
    pub relative_path: String,
    /// 命中/替换次数；恢复操作固定为 0。
    pub matches: usize,
    pub status: EntryStatus,
    /// 失败原因，成功时为 `None`。
    pub error: Option<String>,
}

impl ReportEntry {
    pub(crate) fn success(root: &Path, path: &Path, matches: usize, status: EntryStatus) -> Self {
        Self {
            path: path.to_string_lossy().into_owned(),
            relative_path: display_relative(root, path),
            matches,
            status,
            error: None,
        }
    }

    pub(crate) fn failure(root: &Path, path: &Path, error: String) -> Self {
        Self {
            path: path.to_string_lossy().into_owned(),
            relative_path: display_relative(root, path),
            matches: 0,
            status: EntryStatus::Failed,
            error: Some(error),
        }
    }
}

/// 一次任务的整体结果，可直接序列化给前端。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub operation: Operation,
    /// 根目录。
    pub root: String,
    /// 扫描到的候选文件（或备份）数量。
    pub scanned: usize,
    /// 命中/修改/恢复的文件数量。
    pub changed_files: usize,
    /// 替换次数（恢复操作固定为 0）。
    pub replacements: usize,
    /// 处理失败的文件数量。
    pub failures: usize,
    /// 只包含有命中的文件与失败文件，避免把成千上万个空结果传给前端。
    pub entries: Vec<ReportEntry>,
}

impl Report {
    pub(crate) fn new(operation: Operation, root: &Path) -> Self {
        Self {
            operation,
            root: root.to_string_lossy().into_owned(),
            scanned: 0,
            changed_files: 0,
            replacements: 0,
            failures: 0,
            entries: Vec::new(),
        }
    }
}
