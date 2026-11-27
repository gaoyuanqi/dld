//! 深渊之潮

use serde::{Deserialize, Serialize};

/// 深渊之潮
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ShenYuanZhiChao {
    /// 挑战副本
    pub 深渊秘境: ShenYuanMiJing,
}

/// 深渊秘境
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ShenYuanMiJing {
    /// 兑换副本
    pub 兑换: bool,
    /// 挑战副本
    pub 副本: ShenYuanMiJingCopy,
}

/// 深渊秘境副本
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum ShenYuanMiJingCopy {
    #[serde(rename = "崎岖斗界")]
    #[default]
    崎岖斗界,
    #[serde(rename = "魂渡桥")]
    魂渡桥,
    #[serde(rename = "须臾之河")]
    须臾之河,
    #[serde(rename = "曲镜空洞")]
    曲镜空洞,
    #[serde(rename = "光影迷界")]
    光影迷界,
    #[serde(rename = "吞厄源头")]
    吞厄源头,
    #[serde(rename = "渊秘祭坛")]
    渊秘祭坛,
    #[serde(rename = "古帝遗迹")]
    古帝遗迹,
}

impl ShenYuanMiJingCopy {
    /// 转换为接口参数值
    pub fn api_id(&self) -> &str {
        match self {
            Self::崎岖斗界 => "1",
            Self::魂渡桥 => "2",
            Self::须臾之河 => "3",
            Self::曲镜空洞 => "4",
            Self::光影迷界 => "5",
            Self::吞厄源头 => "6",
            Self::渊秘祭坛 => "7",
            Self::古帝遗迹 => "8",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 副本接口参数映射正确
    #[test]
    fn test_api_id() {
        assert_eq!(ShenYuanMiJingCopy::崎岖斗界.api_id(), "1");
        assert_eq!(ShenYuanMiJingCopy::魂渡桥.api_id(), "2");
        assert_eq!(ShenYuanMiJingCopy::须臾之河.api_id(), "3");
        assert_eq!(ShenYuanMiJingCopy::曲镜空洞.api_id(), "4");
        assert_eq!(ShenYuanMiJingCopy::光影迷界.api_id(), "5");
        assert_eq!(ShenYuanMiJingCopy::吞厄源头.api_id(), "6");
        assert_eq!(ShenYuanMiJingCopy::渊秘祭坛.api_id(), "7");
        assert_eq!(ShenYuanMiJingCopy::古帝遗迹.api_id(), "8");
    }
}
