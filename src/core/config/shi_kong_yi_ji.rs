//! 时空遗迹

use serde::{Deserialize, Serialize};

/// 时空遗迹
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ShiKongYiJiAccount {
    /// 异兽洞窟玩法配置
    pub 异兽洞窟: YiShouDongKu,
}

/// 异兽洞窟玩法配置
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct YiShouDongKu {
    /// 最多打到哪个BOSS
    #[serde(rename = "最多打到哪个BOSS")]
    pub max_boss: YiShouDongKuBoss,
}

/// 异兽洞窟BOSS
#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum YiShouDongKuBoss {
    #[serde(rename = "异兽幼崽")]
    异兽幼崽,
    #[serde(rename = "异兽战士")]
    异兽战士,
    #[serde(rename = "异兽将领")]
    异兽将领,
    #[serde(rename = "异兽元帅")]
    异兽元帅,
    #[serde(rename = "异兽母巢")]
    异兽母巢,
    #[serde(rename = "王级异兽幼崽")]
    王级异兽幼崽,
    #[serde(rename = "王级异兽战士")]
    王级异兽战士,
    #[serde(rename = "王级异兽将领")]
    王级异兽将领,
    #[serde(rename = "王级异兽元帅")]
    王级异兽元帅,
    #[serde(rename = "王级异兽母巢")]
    #[default]
    王级异兽母巢,
    #[serde(rename = "皇级异兽幼崽")]
    皇级异兽幼崽,
    #[serde(rename = "皇级异兽战士")]
    皇级异兽战士,
    #[serde(rename = "皇级异兽将领")]
    皇级异兽将领,
    #[serde(rename = "皇级异兽元帅")]
    皇级异兽元帅,
    #[serde(rename = "皇级异兽母巢")]
    皇级异兽母巢,
    #[serde(rename = "帝级异兽幼崽")]
    帝级异兽幼崽,
    #[serde(rename = "帝级异兽战士")]
    帝级异兽战士,
    #[serde(rename = "帝级异兽将领")]
    帝级异兽将领,
    #[serde(rename = "帝级异兽元帅")]
    帝级异兽元帅,
    #[serde(rename = "帝级异兽母巢")]
    帝级异兽母巢,
}

impl YiShouDongKuBoss {
    /// 转换为接口参数id
    pub(crate) fn api_value(&self) -> u8 {
        match self {
            Self::异兽幼崽 => 1,
            Self::异兽战士 => 2,
            Self::异兽将领 => 3,
            Self::异兽元帅 => 4,
            Self::异兽母巢 => 5,
            Self::王级异兽幼崽 => 6,
            Self::王级异兽战士 => 7,
            Self::王级异兽将领 => 8,
            Self::王级异兽元帅 => 9,
            Self::王级异兽母巢 => 10,
            Self::皇级异兽幼崽 => 11,
            Self::皇级异兽战士 => 12,
            Self::皇级异兽将领 => 13,
            Self::皇级异兽元帅 => 14,
            Self::皇级异兽母巢 => 15,
            Self::帝级异兽幼崽 => 16,
            Self::帝级异兽战士 => 17,
            Self::帝级异兽将领 => 18,
            Self::帝级异兽元帅 => 19,
            Self::帝级异兽母巢 => 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // BOSS 上限读取与接口参数转换
    #[test]
    fn test_max_boss_load_and_api_value() {
        let ku: YiShouDongKu =
            serde_json::from_str(r#"{"最多打到哪个BOSS": "帝级异兽母巢"}"#).unwrap();
        assert_eq!(ku.max_boss, YiShouDongKuBoss::帝级异兽母巢);
        assert_eq!(ku.max_boss.api_value(), 20);
        assert_eq!(YiShouDongKuBoss::异兽幼崽.api_value(), 1);
        assert_eq!(YiShouDongKuBoss::王级异兽母巢.api_value(), 10);
    }
}
