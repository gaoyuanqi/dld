//! 门派账号配置：万年寺与八叶堂的付费选项

use serde::{Deserialize, Serialize};

/// 门派
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MenPai {
    pub 万年寺: WanNianSi,
    pub 八叶堂: BaYeTang,
}

/// 万年寺
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct WanNianSi {
    /// 获得门贡*40，门派强化书*1
    pub 付费高香香炉: bool,
}

/// 八叶堂
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct BaYeTang {
    /// 除了获得门贡*60，还有3点活跃度
    pub 付费同门切磋: bool,
}
