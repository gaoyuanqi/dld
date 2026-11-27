//! 矿洞

use serde::{Deserialize, Serialize};

/// 矿洞
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct KuangDong {
    pub 开启副本: KuangDongFuBen,
}

/// 矿洞副本
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct KuangDongFuBen {
    /// 第一~第五层
    pub 层数: KuangDongFloor,
    /// 简单、普通、困难
    pub 模式: KuangDongMode,
}

/// 矿洞层数
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum KuangDongFloor {
    #[serde(rename = "第一层")]
    F1,
    #[serde(rename = "第二层")]
    F2,
    #[serde(rename = "第三层")]
    F3,
    #[serde(rename = "第四层")]
    F4,
    #[serde(rename = "第五层")]
    F5,
}

impl KuangDongFloor {
    pub(crate) fn api_value(&self) -> &str {
        match self {
            Self::F1 => "1",
            Self::F2 => "2",
            Self::F3 => "3",
            Self::F4 => "4",
            Self::F5 => "5",
        }
    }
}

/// 矿洞模式
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum KuangDongMode {
    #[serde(rename = "简单")]
    Easy,
    #[serde(rename = "普通")]
    Normal,
    #[serde(rename = "困难")]
    Hard,
}

impl KuangDongMode {
    pub(crate) fn api_value(&self) -> &str {
        match self {
            Self::Easy => "1",
            Self::Normal => "2",
            Self::Hard => "3",
        }
    }
}

impl Default for KuangDongFuBen {
    fn default() -> Self {
        Self {
            层数: KuangDongFloor::F1,
            模式: KuangDongMode::Easy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 层数与模式接口参数映射正确
    #[test]
    fn test_api_value() {
        assert_eq!(KuangDongFloor::F1.api_value(), "1");
        assert_eq!(KuangDongFloor::F2.api_value(), "2");
        assert_eq!(KuangDongFloor::F3.api_value(), "3");
        assert_eq!(KuangDongFloor::F4.api_value(), "4");
        assert_eq!(KuangDongFloor::F5.api_value(), "5");
        assert_eq!(KuangDongMode::Easy.api_value(), "1");
        assert_eq!(KuangDongMode::Normal.api_value(), "2");
        assert_eq!(KuangDongMode::Hard.api_value(), "3");
    }
}
