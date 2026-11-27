//! 竞技场

use serde::{Deserialize, Serialize};

/// 竞技场
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JingJiChang {
    /// 是否赛季期每天兑换十个
    pub 兑换河图洛书: bool,
}
