//! Q宠大乐斗个人版代玩辅助
//!
//! 自动执行每日任务，支持多账号管理
//!
//! # 架构
//!
//! ```text
//! cli (命令解析)
//!   -> app (应用编排：登记, 注销, 代玩)
//!      -> dw (玩法层)     core (核心层)
//!         - daledou        - config
//!         - tasks          - client
//!                          - log
//!                          - accounts
//!                          - paths
//! ```
//!
//! # 入口
//!
//! - 可执行文件入口在 `main.rs`
//! - 命令行解析在 [`cli`] 模块
//! - 任务分发见 [`dw::tasks`]

pub mod app;
pub mod cli;
pub mod core;
pub mod dw;
