//! 应用编排层：初始化依赖，登记/注销账号，执行任务
//!
//! [`App`] 是命令行到核心逻辑的桥梁——加载配置、创建 HTTP 客户端，
//! 按并发数调度多账号任务执行，最后统一输出成功/失败统计

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::channel;

use crate::core::accounts::{Account, AccountStore};
use crate::core::client::{self, DaLeDouClient};
use crate::core::config::{AccountConfig, GlobalConfig, UpdatableConfig, get_leaf_value};
use crate::core::log::{
    LOG_CHANNEL_CAPACITY, LogMessage, LogWriter, Logger, cleanup_old_logs, remove_account_logs,
};
use crate::core::paths::Paths;
use crate::dw::daledou::DaLeDou;
use crate::dw::tasks::{self, Task};

/// 应用程序：初始化依赖，编排所有用户命令
pub struct App {
    paths: Paths,
    http_client: Client,
    accounts: AccountStore,
}

impl App {
    /// 初始化应用程序（路径 → HTTP 客户端 → 账号存储）
    pub fn init() -> Result<Self> {
        let paths = Paths::new()?;
        // 每次启动统一收紧权限，覆盖老用户升级前创建的宽松权限文件
        paths.tighten_permissions()?;
        let http_client = client::default_http_client()?;
        let accounts = AccountStore::new(paths.cookies_file());
        Ok(App {
            paths,
            http_client,
            accounts,
        })
    }

    // ── 账号管理 ──

    /// 登记新账号：解析 Cookie → 验证 → 保存 → 创建默认配置
    pub async fn register(&self, raw_cookie: &str) -> Result<()> {
        let account = Account::from_cookie(raw_cookie)?;

        let client = DaLeDouClient::new(&self.http_client, &account.record.cookie);
        client
            .verify_cookie()
            .await
            .with_context(|| format!("验证 {} Cookie 失败", account.qq))?;

        self.accounts.save(&account)?;

        println!("\n{} 登记成功！", account.qq);

        let config_path = self.paths.config_dir().join(format!("{}.json", account.qq));
        AccountConfig::create_default(&config_path)?;

        Ok(())
    }

    /// 注销账号：删除 Cookie → 配置 → 日志
    pub fn unregister(&self, qq: &str) -> Result<()> {
        if self.accounts.remove(qq)? {
            println!("\n{qq} 注销成功！");
        } else {
            println!("\n未找到 QQ {qq} 的账号，无需移除");
        }
        let path = self.paths.config_dir().join(format!("{qq}.json"));
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("删除账号配置失败：{}", path.display()))?;
        }
        remove_account_logs(self.paths.logs_dir(), qq);
        Ok(())
    }

    // ── 信息查询 ──

    /// 打印标准目录结构
    pub fn print_std_dirs(&self) {
        println!("\n标准目录：");
        println!("  {}/", self.paths.data_dir().display());
        println!("    ├── config/               — 账号配置");
        println!("    ├── cookies.json          — 账号 Cookie");
        println!("    ├── global_config.json    — 全局配置");
        println!("    └── logs/                 — 运行日志");
    }

    /// 更新所有配置文件（补充新增字段、删除废弃字段）
    pub fn update_config(&self) -> Result<()> {
        update_and_report::<GlobalConfig>(self.paths.global_config_file())?;
        for qq in &self.accounts.list_qqs()? {
            let path = self.paths.config_dir().join(format!("{qq}.json"));
            update_and_report::<AccountConfig>(&path)?;
        }
        Ok(())
    }

    // ── 任务执行 ──

    /// 执行单个任务
    pub async fn run_task(&self, task: Task, qq: Option<String>) -> Result<()> {
        self.execute(Some(task), qq).await
    }

    /// 执行全部任务
    pub async fn run_all_task(&self, qq: Option<String>) -> Result<()> {
        self.execute(None, qq).await
    }

    /// 执行任务（`task` 为 `None` 表示执行全部）
    async fn execute(&self, task: Option<Task>, qq: Option<String>) -> Result<()> {
        let global_config = Arc::new(GlobalConfig::load(self.paths.global_config_file())?);

        let accounts = match qq {
            Some(q) => vec![self.accounts.find(&q)?],
            None => self.accounts.load_all()?,
        };

        let max_concurrency = global_config.运行时.并发数 as usize;
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let started = Instant::now();

        // 日志通道：各账号任务发送日志，单一 writer 统一输出并落盘
        let (log_tx, log_rx) = channel::<LogMessage>(LOG_CHANNEL_CAPACITY);
        let log_paths = self.paths.clone();
        let log_writer = tokio::spawn(async move {
            LogWriter::new(log_rx, log_paths.logs_dir(), true)
                .run()
                .await;
        });

        let total = accounts.len();
        let handles: Vec<_> = accounts
            .into_iter()
            .map(|acc| {
                let qq = acc.qq.clone();
                let sem = semaphore.clone();
                let http_client = self.http_client.clone();
                let paths = self.paths.clone();
                let task = task.clone();
                let global_config = global_config.clone();
                let log_tx = log_tx.clone();
                let handle = tokio::spawn(async move {
                    let _permit = sem.acquire_owned().await;

                    let config_path = paths.config_dir().join(format!("{}.json", acc.qq));
                    let config = AccountConfig::load(&config_path)?;

                    let logger = Logger::new(paths.logs_dir(), &acc.qq, log_tx)?;
                    let client = DaLeDouClient::new(&http_client, &acc.record.cookie);
                    let d = DaLeDou::new(&acc.qq, client, logger, global_config, config);

                    match &task {
                        Some(t) => Self::run_one(&d, t).await?,
                        None => {
                            for task in Task::all() {
                                Self::run_one(&d, task).await?;
                            }
                        }
                    }
                    Ok::<_, anyhow::Error>(())
                });
                (qq, handle)
            })
            .collect();

        // 等待所有任务完成，收集错误最后统一输出
        let mut failures: Vec<(String, String)> = Vec::new();
        for (qq, handle) in handles {
            match handle.await {
                Ok(Err(e)) => failures.push((qq, format!("{e:#}"))),
                Err(e) => failures.push((qq, format!("任务 panic：{e:#}"))),
                _ => {}
            }
        }

        // 所有账号任务已结束，关闭日志通道并等待日志全部落盘
        drop(log_tx);
        let _ = log_writer.await;

        let elapsed = started.elapsed();
        let succeeded = total - failures.len();
        if failures.is_empty() {
            println!(
                "\n{succeeded}/{total} 个账号全部执行成功，耗时 {:.0?}",
                elapsed
            );
        } else {
            eprintln!(
                "\n{succeeded}/{total} 成功，{} 失败，耗时 {:.0?}：",
                failures.len(),
                elapsed
            );
            for (qq, msg) in &failures {
                eprintln!("  {qq}: {msg}");
            }
        }

        cleanup_old_logs(self.paths.logs_dir(), global_config.运行时.日志保留天数);

        Ok(())
    }

    /// 执行单个任务并检查是否失效或维护
    async fn run_one(d: &DaLeDou, task: &Task) -> Result<()> {
        tasks::run_task(d, task).await;
        if d.is_invalid() {
            d.log("Cookie", "请重新执行命令确认Cookie是否失效");
            anyhow::bail!("请重新执行命令确认Cookie是否失效");
        }
        if d.is_maintenance() {
            d.log("dld", "大乐斗维护中");
            anyhow::bail!("大乐斗维护中");
        }
        Ok(())
    }
}

/// 更新单个配置文件并打印 diff 报告
fn update_and_report<T: UpdatableConfig>(path: &Path) -> Result<()> {
    let default_value = serde_json::to_value(T::default())?;
    let diff = T::update(path)?;
    let has_changes = !diff.added.is_empty() || !diff.removed.is_empty();

    if !diff.added.is_empty() {
        println!("\n新增：");
        for p in &diff.added {
            if let Some(v) = get_leaf_value(&default_value, p) {
                println!("  {p}: {}，默认 {}", value_kind(v), format_default(v));
            }
        }
    }
    if !diff.removed.is_empty() {
        println!("\n移除：");
        for r in &diff.removed {
            println!("  {r}");
        }
    }
    if has_changes {
        println!("\n{}已更新：{}", T::section_title(), path.display());
    }

    Ok(())
}

/// JSON 值类型的中文名
fn value_kind(v: &Value) -> &'static str {
    match v {
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
}

/// 格式化默认值（字符串加引号）
fn format_default(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{s}\""),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => {
            let len = arr.len();
            if len == 0 {
                "[]".to_string()
            } else {
                format!("[{len} 个元素]")
            }
        }
        Value::Object(_) => "{}".to_string(),
    }
}
