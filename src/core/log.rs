//! 账号级别日志记录器：任务发消息到有界通道，单一 [`LogWriter`] 统一落盘
//!
//! 每个 [`Logger`] 实例对应一个 QQ 账号，只负责向日志通道发送消息；
//! [`LogWriter`] 消费通道消息，写入 `<qq>/YYYY-MM-DD.log` 文件，
//! 是否同步输出到控制台由开关控制；
//! 同时提供 [`cleanup_old_logs`] 按保留天数自动清理过期日志
//!
//! 文件写入为每次打开→追加→关闭

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use chrono::{NaiveDate, NaiveDateTime};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};

/// 日志通道容量：足以容纳所有任务的日志突发，超出时丢弃新日志
pub(crate) const LOG_CHANNEL_CAPACITY: usize = 1024;

/// 待写入的日志消息
#[derive(Debug)]
pub(crate) struct LogMessage {
    pub(crate) qq: String,
    pub(crate) now: NaiveDateTime,
    pub(crate) task_name: String,
    pub(crate) result: String,
}

/// 账号级别日志记录器：向日志通道发送消息
pub struct Logger {
    tx: Sender<LogMessage>,
    qq: String,
}

impl Logger {
    /// 创建日志记录器
    ///
    /// 创建前校验日志目录可写，消息由 [`LogWriter`] 统一落盘，
    /// 是否同步输出到控制台由 [`LogWriter::new`] 的开关控制
    pub(crate) fn new(logs_dir: &Path, qq: &str, tx: Sender<LogMessage>) -> Result<Self> {
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
            tx,
            qq: qq.to_string(),
        })
    }

    /// 记录日志：发送消息到日志通道（已满时丢弃并提示、关闭时静默，不影响任务执行）
    pub fn log(&self, now: NaiveDateTime, task_name: &str, result: &str) {
        let msg = LogMessage {
            qq: self.qq.clone(),
            now,
            task_name: task_name.to_string(),
            result: result.to_string(),
        };
        match self.tx.try_send(msg) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                eprintln!("[{}] 日志通道已满，丢弃一条日志", self.qq);
            }
            Err(TrySendError::Closed(_)) => {}
        }
    }
}

/// 日志写入器：消费日志通道，控制台输出并落盘
pub(crate) struct LogWriter {
    rx: Receiver<LogMessage>,
    logs_dir: PathBuf,
    console: bool,
}

impl LogWriter {
    /// 创建日志写入器（`console` 控制是否同步输出到控制台）
    pub(crate) fn new(rx: Receiver<LogMessage>, logs_dir: &Path, console: bool) -> Self {
        Self {
            rx,
            logs_dir: logs_dir.to_path_buf(),
            console,
        }
    }

    /// 消费日志消息直到通道关闭，随后正常退出
    pub(crate) async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            if self.console {
                println!(
                    "{}",
                    format_console_line(
                        &msg.now.format("%Y-%m-%d %H:%M:%S").to_string(),
                        &msg.qq,
                        &msg.task_name,
                        &msg.result,
                    )
                );
            }
            self.write(&msg);
        }
    }

    /// 写入日志行到文件（每次打开→追加→关闭）
    fn write(&self, msg: &LogMessage) {
        let line = format_file_line(
            &msg.now.format("%H:%M:%S").to_string(),
            &msg.task_name,
            &msg.result,
        );
        let path = log_file_path(&self.logs_dir, &msg.qq, msg.now.date());

        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                let _ = writeln!(f, "{}", line);
            }
            Err(e) => {
                eprintln!("[{}] 无法打开日志文件 {}：{}", msg.qq, path.display(), e);
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

    use chrono::{Local, NaiveDate};

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

    // 多条日志经通道落盘，顺序与发送顺序一致
    #[tokio::test]
    async fn test_writer_persists_logs_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel(LOG_CHANNEL_CAPACITY);
        let writer = LogWriter::new(rx, dir.path(), false);

        let logger = Logger::new(dir.path(), "123456", tx.clone()).unwrap();
        let now = Local::now().naive_local();
        logger.log(now, "开始乐斗", "完成");
        logger.log(now, "帮派任务", "领取成功");

        // 关闭通道，writer 消费完所有消息后退出
        drop(logger);
        drop(tx);
        writer.run().await;

        let content = fs::read_to_string(log_file_path(dir.path(), "123456", now.date())).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("开始乐斗：完成"));
        assert!(lines[1].contains("帮派任务：领取成功"));
    }

    // 通道满时丢弃新日志，不阻塞任务
    #[tokio::test]
    async fn test_channel_full_drops_new_logs() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel(2);
        let writer = LogWriter::new(rx, dir.path(), false);

        let logger = Logger::new(dir.path(), "123456", tx.clone()).unwrap();
        let now = Local::now().naive_local();
        logger.log(now, "一", "1");
        logger.log(now, "二", "2");
        logger.log(now, "三", "3"); // 通道已满，被丢弃

        drop(logger);
        drop(tx);
        writer.run().await;

        let content = fs::read_to_string(log_file_path(dir.path(), "123456", now.date())).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("一：1"));
        assert!(lines[1].contains("二：2"));
    }

    // 通道积压被消费后恢复，后续日志正常发送
    #[tokio::test]
    async fn test_channel_recovers_after_consume() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);
        let logger = Logger::new(dir.path(), "123456", tx).unwrap();
        let now = Local::now().naive_local();

        logger.log(now, "一", "1");
        logger.log(now, "二", "2");
        let first = rx.recv().await.unwrap();
        assert_eq!(first.task_name, "一");

        logger.log(now, "三", "3"); // 消费后腾出空间，发送成功

        let second = rx.recv().await.unwrap();
        let third = rx.recv().await.unwrap();
        assert_eq!(second.task_name, "二");
        assert_eq!(third.task_name, "三");
        drop(logger);
        drop(rx);
    }

    // 不同账号的日志写入各自文件
    #[tokio::test]
    async fn test_writer_routes_logs_by_qq() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel(LOG_CHANNEL_CAPACITY);
        let writer = LogWriter::new(rx, dir.path(), false);

        let logger_a = Logger::new(dir.path(), "111", tx.clone()).unwrap();
        let logger_b = Logger::new(dir.path(), "222", tx.clone()).unwrap();
        let now = Local::now().naive_local();
        logger_a.log(now, "开始乐斗", "完成");
        logger_b.log(now, "帮派任务", "领取成功");

        drop(logger_a);
        drop(logger_b);
        drop(tx);
        writer.run().await;

        let file_a = log_file_path(dir.path(), "111", now.date());
        let file_b = log_file_path(dir.path(), "222", now.date());
        assert!(file_a.exists());
        assert!(file_b.exists());
        assert!(
            fs::read_to_string(&file_a)
                .unwrap()
                .contains("开始乐斗：完成")
        );
        assert!(
            fs::read_to_string(&file_b)
                .unwrap()
                .contains("帮派任务：领取成功")
        );
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
