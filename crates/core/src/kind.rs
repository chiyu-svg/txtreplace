use std::path::Path;

use serde::{Deserialize, Serialize};

/// 支持处理的文件类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Txt,
    Docx,
    Xlsx,
}

impl FileKind {
    /// 全部支持的格式，供 CLI 与默认行为使用。
    pub const ALL: [Self; 3] = [Self::Txt, Self::Docx, Self::Xlsx];

    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;

        if extension.eq_ignore_ascii_case("txt") {
            Some(Self::Txt)
        } else if extension.eq_ignore_ascii_case("docx") {
            Some(Self::Docx)
        } else if extension.eq_ignore_ascii_case("xlsx") {
            Some(Self::Xlsx)
        } else {
            None
        }
    }
}
