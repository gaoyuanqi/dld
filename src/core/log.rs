//! 账号级别日志记录器：控制台即时输出 + 文件同步落盘
//!
//! 每个 [`Logger`] 实例对应一个 QQ 账号，日志按天写入 `<qq>.log` 文件，
//! 同时提供 [`cleanup_old_logs`] 按保留天数自动清理过期日志
//!
//! 每个 `Logger` 实例对应一个 QQ 账号
//!
//! 文件写入为每次打开→追加→关闭

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use chrono::{NaiveDate, NaiveDateTime};

/// 账号级别日志记录器
pub struct Logger {
    logs_dir: PathBuf,
    qq: String,
}

impl Logger {
    /// 创建日志记录器
    ///
    /// 日志文件路径：`<logs_dir>/<qq>/YYYY-MM-DD.log`
    pub fn new(logs_dir: &Path, qq: &str) -> Result<Self> {
        let logs_dir = logs_dir.to_path_buf();
        let log_dir = logs_dir.join(qq);

        // 确保日志目录存在
        if let Err(e) = fs::create_dir_all(&log_dir) {
            bail!("无法创建日志目录：{}: {e}", log_dir.display());
        }

        // 验证目录是否可写
        let test_file = log_dir.join(".write_test");
        if let Err(e) = fs::write(&test_file, b"test") {
            bail!("日志目录不可写：{}: {e}", log_dir.display());
        }
        let _ = fs::remove_file(&test_file);

        Ok(Self {
            logs_dir,
            qq: qq.to_string(),
        })
    }

    /// 记录日志：控制台输出 + 文件写入
    pub fn log(&self, now: NaiveDateTime, task_name: &str, result: &str) {
        println!(
            "{}",
            format_console_line(
                &now.format("%Y-%m-%d %H:%M:%S").to_string(),
                &self.qq,
                task_name,
                result,
            )
        );
        self.send(now, task_name, result);
    }

    /// 写入日志行到文件（每次打开→追加→关闭）
    fn send(&self, now: NaiveDateTime, task_name: &str, result: &str) {
        let line = format_file_line(&now.format("%H:%M:%S").to_string(), task_name, result);
        let path = log_file_path(&self.logs_dir, &self.qq, now.date());

        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                let _ = writeln!(f, "{}", line);
            }
            Err(e) => {
                eprintln!("[{}] 无法打开日志文件 {}：{}", self.qq, path.display(), e);
            }
        }
    }
}

/// 生成日志行（文件格式）：`14:36:00 | 任务名称：结果`
fn format_file_line(time: &str, task_name: &str, result: &str) -> String {
    format!("{time} | {task_name}：{result}")
}

/// 生成日志行（控制台格式）：`2026-06-15 14:36:00 | QQ | 任务名称：结果`
fn format_console_line(time: &str, qq: &str, task_name: &str, result: &str) -> String {
    format!("{time} | {qq} | {task_name}：{result}")
}

/// 根据日期生成日志文件路径：`<logs_dir>/<qq>/YYYY-MM-DD.log`
fn log_file_path(logs_dir: &Path, qq: &str, date: NaiveDate) -> PathBuf {
    logs_dir
        .join(qq)
        .join(format!("{}.log", date.format("%Y-%m-%d")))
}

/// 删除指定 QQ 的全部日志，目录不存在则无操作
pub fn remove_account_logs(logs_dir: &Path, qq: &str) {
    let dir = logs_dir.join(qq);
    let _ = fs::remove_dir_all(&dir);
}

/// 清理超过保留天数的旧日志文件
///
/// 日志目录结构：`<logs_dir>/<qq>/YYYY-MM-DD.log`
pub fn cleanup_old_logs(logs_dir: &Path, retention_days: u8) {
    do_cleanup(logs_dir, retention_days);
}

fn do_cleanup(logs_dir: &Path, retention_days: u8) {
    let today = chrono::Local::now().date_naive();
    let cutoff = today - chrono::TimeDelta::days((retention_days - 1) as i64);

    let entries = match fs::read_dir(logs_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let log_entries = match fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for f in log_entries.flatten() {
            let file_path = f.path();
            let Some(name) = file_path.file_stem().and_then(|n| n.to_str()) else {
                continue;
            };

            // 解析文件名中的日期：YYYY-MM-DD
            let Ok(date) = NaiveDate::parse_from_str(name, "%Y-%m-%d") else {
                continue;
            };

            if date < cutoff {
                let _ = fs::remove_file(&file_path);
            }
        }

        // 清理空目录
        let _ = fs::remove_dir(&path);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::NaiveDate;

    use super::*;

    // 日志文件行格式
    #[test]
    fn test_format_file_line() {
        let line = format_file_line("14:30:00", "开始乐斗", "完成");
        assert_eq!(line, "14:30:00 | 开始乐斗：完成");
    }

    // 控制台行格式
    #[test]
    fn test_format_console_line() {
        let line = format_console_line("2026-06-15 14:30:00", "123456", "开始乐斗", "完成");
        assert_eq!(line, "2026-06-15 14:30:00 | 123456 | 开始乐斗：完成");
    }

    // 日志文件路径拼接
    #[test]
    fn test_log_file_path() {
        let dir = Path::new("/tmp/dld");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let path = log_file_path(dir, "123456", date);
        assert_eq!(path, PathBuf::from("/tmp/dld/123456/2026-06-15.log"));
    }

    // 保留 N 天：今天 + N-1 天前之内的日志保留，更旧的删除
    #[test]
    fn test_cleanup_old_logs() {
        let dir = tempfile::tempdir().unwrap();
        let logs_dir = dir.path();

        let today = chrono::Local::now().date_naive();
        let old_date = today - chrono::TimeDelta::days(10);
        let recent_date = today - chrono::TimeDelta::days(2);

        let qq_dir = logs_dir.join("123456");
        fs::create_dir_all(&qq_dir).unwrap();

        // 旧日志 — 应被删除
        let old_name = format!("{}.log", old_date.format("%Y-%m-%d"));
        fs::write(qq_dir.join(&old_name), b"old").unwrap();

        // 近期日志 — 应保留（含边界：2 天前 = 今日 + 前 2 天 = 共 3 天）
        let recent_name = format!("{}.log", recent_date.format("%Y-%m-%d"));
        fs::write(qq_dir.join(&recent_name), b"recent").unwrap();

        do_cleanup(logs_dir, 3);

        assert!(!qq_dir.join(&old_name).exists());
        assert!(qq_dir.join(&recent_name).exists());
    }

    // 边界：保留 1 天，昨天的日志也会被删除
    #[test]
    fn test_cleanup_old_logs_retention_1_deletes_yesterday() {
        let dir = tempfile::tempdir().unwrap();
        let logs_dir = dir.path();

        let today = chrono::Local::now().date_naive();
        let yesterday = today - chrono::TimeDelta::days(1);

        let qq_dir = logs_dir.join("123456");
        fs::create_dir_all(&qq_dir).unwrap();

        let name = format!("{}.log", yesterday.format("%Y-%m-%d"));
        fs::write(qq_dir.join(&name), b"yesterday").unwrap();

        do_cleanup(logs_dir, 1);

        assert!(!qq_dir.join(&name).exists());
    }

    // 空目录不崩溃
    #[test]
    fn test_cleanup_old_logs_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        do_cleanup(dir.path(), 30);
    }
}
