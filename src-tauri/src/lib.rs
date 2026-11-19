//! Tauri 命令层。
//!
//! 这一层只做三件事：接收前端参数、把阻塞的文件操作丢到后台线程、把核心层错误转成字符串。
//! 所有替换逻辑都在 `txtreplace-core` 中，命令层不包含业务实现。

use std::path::PathBuf;

use txtreplace_core::{Error, FileKind, Mode, Report};

#[tauri::command]
async fn preview_replace(
    directory: String,
    needle: String,
    replacement: String,
    kinds: Vec<FileKind>,
) -> Result<Report, String> {
    run_blocking(move || {
        txtreplace_core::replace(
            &PathBuf::from(directory),
            &needle,
            &replacement,
            Mode::Preview,
            &kinds,
        )
    })
    .await
}

#[tauri::command]
async fn apply_replace(
    directory: String,
    needle: String,
    replacement: String,
    kinds: Vec<FileKind>,
) -> Result<Report, String> {
    run_blocking(move || {
        txtreplace_core::replace(
            &PathBuf::from(directory),
            &needle,
            &replacement,
            Mode::Apply,
            &kinds,
        )
    })
    .await
}

#[tauri::command]
async fn restore_backups(directory: String) -> Result<Report, String> {
    run_blocking(move || txtreplace_core::restore(&PathBuf::from(directory))).await
}

async fn run_blocking<F>(task: F) -> Result<Report, String>
where
    F: FnOnce() -> Result<Report, Error> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| format!("后台任务异常退出：{error}"))?
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            preview_replace,
            apply_replace,
            restore_backups
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
