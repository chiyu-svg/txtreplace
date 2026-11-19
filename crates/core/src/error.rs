use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use zip::result::ZipError;

/// 核心层错误。
///
/// 应用层（CLI / Tauri command）负责把它转换成用户可见的提示或退出码。
#[derive(Debug, Error)]
pub enum Error {
    #[error("旧文本不能为空")]
    EmptyNeedle,

    #[error("`{}` 不是目录", .0.display())]
    NotDirectory(PathBuf),

    #[error("操作 `{}` 失败: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },

    #[error("解析 Office 文档 `{}` 失败: {source}", path.display())]
    Zip { path: PathBuf, source: ZipError },

    #[error("`{}` 中的 `{entry}` 不是合法 UTF-8，为避免损坏已跳过", path.display())]
    Encoding { path: PathBuf, entry: String },
}

impl Error {
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub(crate) fn zip(path: &Path, source: ZipError) -> Self {
        Self::Zip {
            path: path.to_path_buf(),
            source,
        }
    }
}
