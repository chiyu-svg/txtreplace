//! 文本替换核心逻辑。
//!
//! 这一层只依赖标准库与 `zip`，不感知 Tauri、命令行或任何界面，
//! 因此可以被桌面端、CLI、测试甚至以后的 Web 端直接复用。

mod engine;
mod error;
mod fsutil;
mod report;

pub use engine::{Mode, replace, restore};
pub use error::Error;
pub use report::{EntryStatus, Operation, Report, ReportEntry};
