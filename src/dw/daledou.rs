use std::sync::Arc;

use anyhow::{Result, bail};
use chrono::Local;
use serde::de::DeserializeOwned;

use crate::core::client::DaLeDouClient;
use crate::core::config::{AccountConfig, GlobalConfig};
use crate::core::log::Logger;

/// 任务执行上下文，绑定单个账号的配置、HTTP 客户端和日志
///
/// 所有任务函数都通过 `&DaLeDou` 获取配置、发起请求、记录日志
/// 由 [`App`](crate::app::App) 在任务执行前创建，任务结束后销毁
pub struct DaLeDou {
    /// 账号 QQ 号
    qq: String,
    /// HTTP 客户端（绑定该账号的 cookie）
    client: DaLeDouClient,
    logger: Logger,
    /// 全局配置（只读，多账号共享）
    global_config: Arc<GlobalConfig>,
    /// 账号配置（每个 QQ 独享）
    config: AccountConfig,
}

impl DaLeDou {
    /// 创建任务执行上下文
    pub fn new(
        qq: &str,
        client: DaLeDouClient,
        logger: Logger,
        global_config: Arc<GlobalConfig>,
        config: AccountConfig,
    ) -> Self {
        Self {
            qq: qq.to_string(),
            client,
            logger,
            global_config,
            config,
        }
    }

    /// 发起 GET 请求并反序列化为目标类型
    pub async fn get<T: DeserializeOwned>(&self, cmd_path: &str) -> Result<T> {
        let json = self.client.get(cmd_path).await?;
        let data = match serde_json::from_value(json) {
            Ok(v) => v,
            Err(e) => bail!("解析响应失败 [{cmd_path}]: {e}"),
        };
        Ok(data)
    }

    /// 验证当前账号 Cookie 是否有效
    pub async fn verify_cookie(&self) -> Result<()> {
        self.client.verify_cookie().await
    }

    /// 返回账号 QQ 号
    pub fn qq(&self) -> &str {
        &self.qq
    }

    /// 记录任务日志
    pub fn log(&self, task_name: &str, msg: &str) {
        self.logger.log(Local::now().naive_local(), task_name, msg);
    }

    /// Cookie 是否已失效
    pub fn is_invalid(&self) -> bool {
        self.client.is_invalid()
    }

    /// 大乐斗是否处于维护状态
    pub fn is_maintenance(&self) -> bool {
        self.client.is_maintenance()
    }

    /// 返回全局配置（只读引用）
    pub fn global_config(&self) -> &GlobalConfig {
        &self.global_config
    }

    /// 返回账号配置（只读引用）
    pub fn config(&self) -> &AccountConfig {
        &self.config
    }
}
