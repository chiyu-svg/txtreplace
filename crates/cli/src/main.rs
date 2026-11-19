//! txtreplace 命令行入口。
//!
//! 只负责解析参数、调用核心层、打印结果，不包含任何替换逻辑。

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use txtreplace_core::{EntryStatus, Error, Mode, Report, replace, restore};

const USAGE: &str = "\
用法:
  txtreplace <目录> <旧文本> <新文本> [--write]   统计并可选执行替换
  txtreplace --restore <目录>                     用 .bak 备份回滚

说明:
  · 支持 .txt / .docx / .xlsx，递归扫描子目录；
  · 默认只预览命中数量，加 --write 才真正写入文件；
  · 写入前会自动备份为 <原文件名>.bak（只保留最近一份）；
  · 有文件处理失败时退出码为 1。";

fn main() -> ExitCode {
    match Cli::parse(env::args().skip(1)) {
        Ok(Some(cli)) => match cli.run() {
            Ok(report) if report.failures == 0 => ExitCode::SUCCESS,
            Ok(_) => ExitCode::FAILURE,
            Err(error) => {
                eprintln!("错误: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(None) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("错误: {message}");
            eprintln!();
            println!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

enum Cli {
    Replace {
        dir: PathBuf,
        old: String,
        new: String,
        write: bool,
    },
    Restore {
        dir: PathBuf,
    },
}

impl Cli {
    /// 解析参数；`Ok(None)` 表示用户请求帮助信息。
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Option<Self>, String> {
        let mut positional = Vec::new();
        let mut write = false;
        let mut restore = false;

        for arg in args {
            match arg.as_str() {
                "--write" => write = true,
                "--restore" => restore = true,
                "-h" | "--help" => return Ok(None),
                _ if arg.starts_with("--") => {
                    return Err(format!("未知参数 `{arg}`，可用 `--help` 查看用法"));
                }
                _ => positional.push(arg),
            }
        }

        if restore {
            if write {
                return Err("`--restore` 不能与 `--write` 同时使用".into());
            }
            let [dir] = <[String; 1]>::try_from(positional)
                .map_err(|_| "`--restore` 需要且只需要一个目录参数")?;
            return Ok(Some(Self::Restore { dir: dir.into() }));
        }

        let [dir, old, new] = <[String; 3]>::try_from(positional)
            .map_err(|_| "需要 3 个参数：<目录> <旧文本> <新文本>")?;

        Ok(Some(Self::Replace {
            dir: dir.into(),
            old,
            new,
            write,
        }))
    }

    fn run(self) -> Result<Report, Error> {
        match self {
            Self::Replace {
                dir,
                old,
                new,
                write,
            } => {
                let mode = if write { Mode::Apply } else { Mode::Preview };
                let report = replace(&dir, &old, &new, mode)?;
                print_replace_report(&report, &old, &new, write);
                Ok(report)
            }
            Self::Restore { dir } => {
                let report = restore(&dir)?;
                print_restore_report(&report);
                Ok(report)
            }
        }
    }
}

fn print_replace_report(report: &Report, old: &str, new: &str, write: bool) {
    println!("目录: {}", report.root);
    println!("替换: {old:?} -> {new:?}");

    print_entries(report);

    println!("{:-<48}", "");
    if write {
        println!(
            "扫描 {} 个文件：已替换 {} 个，共 {} 处",
            report.scanned, report.changed_files, report.replacements
        );
    } else {
        println!(
            "扫描 {} 个文件：命中 {} 个，共 {} 处（预览，加 --write 才会写入）",
            report.scanned, report.changed_files, report.replacements
        );
    }
    print_failures(report);
}

fn print_restore_report(report: &Report) {
    print_entries(report);

    println!("{:-<48}", "");
    println!("回撤完成：恢复 {} 个文件", report.changed_files);
    print_failures(report);
}

fn print_entries(report: &Report) {
    for entry in &report.entries {
        if entry.status == EntryStatus::Failed {
            eprintln!("  ✗ {}", entry.error.as_deref().unwrap_or("未知错误"));
        } else {
            println!("{:>5} 处  {}", entry.matches, entry.path);
        }
    }
}

fn print_failures(report: &Report) {
    if report.failures > 0 {
        println!("失败 {} 个文件，详见上方错误信息", report.failures);
    }
}
