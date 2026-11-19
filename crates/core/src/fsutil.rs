use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::kind::FileKind;

/// 备份后缀：`a.txt` -> `a.txt.bak`。
pub(crate) const BACKUP_SUFFIX: &str = ".bak";

pub(crate) fn ensure_dir(dir: &Path) -> Result<(), Error> {
    match fs::metadata(dir) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(Error::NotDirectory(dir.to_path_buf())),
        Err(source) => Err(Error::io(dir, source)),
    }
}

pub(crate) fn collect_target_files(dir: &Path, kinds: &[FileKind]) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    walk(dir, &mut files, &|path| {
        FileKind::from_path(path).is_some_and(|kind| kinds.contains(&kind))
    })?;
    files.sort();
    Ok(files)
}

pub(crate) fn collect_backups(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut backups = Vec::new();
    walk(dir, &mut backups, &is_backup)?;
    backups.sort();
    Ok(backups)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, keep: &impl Fn(&Path) -> bool) -> Result<(), Error> {
    for entry in fs::read_dir(dir).map_err(|error| Error::io(dir, error))? {
        let entry = entry.map_err(|error| Error::io(dir, error))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| Error::io(&path, error))?;

        if file_type.is_dir() {
            walk(&path, out, keep)?;
        } else if file_type.is_file() && keep(&path) {
            out.push(path);
        }
    }

    Ok(())
}

/// 相对路径统一使用 `/` 分隔，方便前端展示。
pub(crate) fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// 写入前备份：`path` -> `path.bak`（覆盖式，只保留最近一份）。
pub(crate) fn backup_file(path: &Path) -> Result<(), Error> {
    let backup = backup_path(path);
    fs::copy(path, &backup).map_err(|error| Error::io(&backup, error))?;
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(OsStr::to_os_string)
        .unwrap_or_default();
    name.push(BACKUP_SUFFIX);
    path.with_file_name(name)
}

/// `a.txt.bak` -> `a.txt`
pub(crate) fn restore_target(backup: &Path) -> Option<PathBuf> {
    let name = backup.file_name()?.to_str()?;
    let original = name.strip_suffix(BACKUP_SUFFIX)?;
    Some(backup.with_file_name(original))
}

fn is_backup(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.len() > BACKUP_SUFFIX.len() && name.ends_with(BACKUP_SUFFIX))
}
