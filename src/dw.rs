//! 玩法层：任务调度与各玩法任务的实现
//!
//! # 主要类型
//!
//! - [`daledou::DaLeDou`] — 任务执行上下文，绑定单个账号的配置和 HTTP 客户端
//! - [`tasks::Task`] — 所有可执行任务的枚举
//! - [`tasks::run_task`] — 根据任务枚举分发执行

pub mod daledou;
pub mod tasks;
