use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use zip::read::ZipArchive;
use zip::write::{FileOptions, ZipWriter};

use crate::error::Error;
use crate::fsutil::{
    Kind, backup_file, collect_backups, collect_target_files, ensure_dir, restore_target,
};
use crate::report::{EntryStatus, Operation, Report, ReportEntry};

/// 替换工作模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 只统计命中数量，不修改文件。
    Preview,
    /// 统计并写入，写入前生成 `.bak` 备份。
    Apply,
}

/// 递归处理 `root` 下的 `.txt` / `.docx` / `.xlsx`。
///
/// 单个文件失败不会中断整体任务，失败信息会记录在报告里；
/// 只有参数非法或根目录不可读时才返回 `Err`。
pub fn replace(root: &Path, needle: &str, replacement: &str, mode: Mode) -> Result<Report, Error> {
    if needle.is_empty() {
        return Err(Error::EmptyNeedle);
    }
    ensure_dir(root)?;

    let files = collect_target_files(root)?;
    let write = mode == Mode::Apply;
    let (operation, status) = if write {
        (Operation::Replace, EntryStatus::Written)
    } else {
        (Operation::Preview, EntryStatus::Previewed)
    };

    let mut report = Report::new(operation, root);
    report.scanned = files.len();

    for path in &files {
        match replace_in_file(path, needle, replacement, write) {
            Ok(0) => {}
            Ok(count) => {
                report.changed_files += 1;
                report.replacements += count;
                report
                    .entries
                    .push(ReportEntry::success(root, path, count, status));
            }
            Err(error) => {
                report.failures += 1;
                report
                    .entries
                    .push(ReportEntry::failure(root, path, error.to_string()));
            }
        }
    }

    Ok(report)
}

/// 把目录下所有 `.bak` 备份恢复回原文件，成功后删除备份。
pub fn restore(root: &Path) -> Result<Report, Error> {
    ensure_dir(root)?;

    let backups = collect_backups(root)?;

    let mut report = Report::new(Operation::Restore, root);
    report.scanned = backups.len();

    for backup in &backups {
        let Some(original) = restore_target(backup) else {
            continue;
        };

        match restore_one(backup, &original) {
            Ok(()) => {
                report.changed_files += 1;
                report.entries.push(ReportEntry::success(
                    root,
                    &original,
                    0,
                    EntryStatus::Restored,
                ));
            }
            Err(error) => {
                report.failures += 1;
                report
                    .entries
                    .push(ReportEntry::failure(root, &original, error.to_string()));
            }
        }
    }

    Ok(report)
}

fn restore_one(backup: &Path, original: &Path) -> Result<(), Error> {
    fs::copy(backup, original).map_err(|error| Error::io(original, error))?;
    fs::remove_file(backup).map_err(|error| Error::io(backup, error))?;
    Ok(())
}

fn replace_in_file(
    path: &Path,
    needle: &str,
    replacement: &str,
    write: bool,
) -> Result<usize, Error> {
    match Kind::from_path(path) {
        Some(Kind::Text) => replace_in_text(path, needle, replacement, write),
        Some(Kind::Docx) => replace_in_zip(path, needle, replacement, write, is_docx_part),
        Some(Kind::Xlsx) => replace_in_zip(path, needle, replacement, write, is_xlsx_part),
        None => Ok(0),
    }
}

/// `.txt`：整文件读取、统计后按需写回。
fn replace_in_text(
    path: &Path,
    needle: &str,
    replacement: &str,
    write: bool,
) -> Result<usize, Error> {
    let content = fs::read_to_string(path).map_err(|error| Error::io(path, error))?;
    let count = content.matches(needle).count();

    if count > 0 && write {
        backup_file(path)?;
        fs::write(path, content.replace(needle, replacement))
            .map_err(|error| Error::io(path, error))?;
    }

    Ok(count)
}

/// `.docx` / `.xlsx`：本质是 ZIP，只对指定的 XML 部件做文本替换。
fn replace_in_zip(
    path: &Path,
    needle: &str,
    replacement: &str,
    write: bool,
    is_text_part: fn(&str) -> bool,
) -> Result<usize, Error> {
    let file = File::open(path).map_err(|error| Error::io(path, error))?;
    let mut archive = ZipArchive::new(file).map_err(|error| Error::zip(path, error))?;

    // 先把整个包读进内存，并记录每个条目的元信息，保证重写后结构一致。
    let mut entries = Vec::with_capacity(archive.len());
    let mut replacements = 0;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| Error::zip(path, error))?;

        let name = entry.name().to_owned();
        let is_dir = entry.is_dir();
        let mut options = FileOptions::default()
            .compression_method(entry.compression())
            .last_modified_time(entry.last_modified());
        if let Some(mode) = entry.unix_mode() {
            options = options.unix_permissions(mode);
        }

        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .map_err(|error| Error::io(path, error))?;

        if !is_dir && is_text_part(&name) {
            let text = std::str::from_utf8(&data).map_err(|_| Error::Encoding {
                path: path.to_path_buf(),
                entry: name.clone(),
            })?;
            replacements += text.matches(needle).count();
            data = text.replace(needle, replacement).into_bytes();
        }

        entries.push(ZipEntry {
            name,
            options,
            data,
            is_dir,
        });
    }

    // Windows 上必须先关闭读句柄，才能替换同一个文件。
    drop(archive);

    if replacements == 0 || !write {
        return Ok(replacements);
    }

    backup_file(path)?;
    rewrite_zip(path, &entries)?;
    Ok(replacements)
}

struct ZipEntry {
    name: String,
    options: FileOptions,
    data: Vec<u8>,
    is_dir: bool,
}

/// 先写同目录下的临时文件，成功后再替换原文件，避免写一半留下损坏的压缩包。
fn rewrite_zip(path: &Path, entries: &[ZipEntry]) -> Result<(), Error> {
    let mut tmp_name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    tmp_name.push(format!(".{}.tmp", std::process::id()));
    let tmp = path.with_file_name(tmp_name);

    let result = write_zip(&tmp, entries)
        .and_then(|()| fs::rename(&tmp, path).map_err(|error| Error::io(path, error)));

    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }

    result
}

fn write_zip(path: &Path, entries: &[ZipEntry]) -> Result<(), Error> {
    let file = File::create(path).map_err(|error| Error::io(path, error))?;
    let mut writer = ZipWriter::new(file);

    for entry in entries {
        if entry.is_dir {
            writer
                .add_directory(entry.name.as_str(), entry.options)
                .map_err(|error| Error::zip(path, error))?;
        } else {
            writer
                .start_file(entry.name.as_str(), entry.options)
                .map_err(|error| Error::zip(path, error))?;
            writer
                .write_all(&entry.data)
                .map_err(|error| Error::io(path, error))?;
        }
    }

    writer.finish().map_err(|error| Error::zip(path, error))?;
    Ok(())
}

/// `.docx` 的正文（页眉 / 页脚 / 批注暂不处理）。
fn is_docx_part(name: &str) -> bool {
    name == "word/document.xml"
}

/// `.xlsx` 的共享字符串表与各工作表。
fn is_xlsx_part(name: &str) -> bool {
    name == "xl/sharedStrings.xml"
        || (name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml"))
}
