//! 核心层：配置管理、HTTP 客户端、日志、账号管理
//!
//! # 主要类型
//!
//! - [`config`] — 全局配置与账号配置的结构体定义及校验
//! - [`client`] — 大乐斗 HTTP 客户端，处理 GBK 解码、Cookie 校验
//! - [`log`] — 任务日志写入与过期清理
//! - [`accounts`] — 多账号登记、注销、查询
//! - [`paths`] — 数据目录路径计算

pub mod accounts;
pub mod client;
pub mod config;
pub mod log;
pub mod paths;
