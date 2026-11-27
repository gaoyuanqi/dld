//! 江湖长梦

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 江湖长梦
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JiangHuChangMeng {
    pub 副本: ChangMengCopy,
    pub 兑换上限: ChangMengExchange,
}

impl JiangHuChangMeng {
    /// 根据 copy_name 返回执行次数上限，未配置的副本返回 0
    pub fn limit(&self, copy_name: &str) -> u32 {
        self.副本.limit(copy_name)
    }
}

/// 长梦副本
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ChangMengCopy {
    /// 执行次数上限，0~200
    pub 柒承的忙碌日常: u32,
    /// 执行次数上限，0~200
    pub 倚天屠龙归我心: u32,
    /// 执行次数上限，0~200
    pub 绝世秘籍之争: u32,
    /// 执行次数上限，0~200
    pub 桃花自古笑春风: u32,
    /// 执行次数上限，0~200
    pub 战乱襄阳: u32,
    /// 执行次数上限，0~200
    pub 天涯浪子: u32,
    /// 执行次数上限，0~200
    pub 全真古墓意难平: u32,
    /// 执行次数上限，0~200
    pub 南海有岛名侠客: u32,
}

impl ChangMengCopy {
    /// 根据 copy_name 返回执行次数上限，未配置的副本返回 0
    pub fn limit(&self, copy_name: &str) -> u32 {
        match copy_name {
            "柒承的忙碌日常" => self.柒承的忙碌日常,
            "倚天屠龙归我心" => self.倚天屠龙归我心,
            "绝世秘籍之争" => self.绝世秘籍之争,
            "桃花自古笑春风" => self.桃花自古笑春风,
            "战乱襄阳" => self.战乱襄阳,
            "天涯浪子" => self.天涯浪子,
            "全真古墓意难平" => self.全真古墓意难平,
            "南海有岛名侠客" => self.南海有岛名侠客,
            _ => 0,
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        validate_range!("江湖长梦.副本.柒承的忙碌日常", self.柒承的忙碌日常, 0, 200);
        validate_range!("江湖长梦.副本.倚天屠龙归我心", self.倚天屠龙归我心, 0, 200);
        validate_range!("江湖长梦.副本.绝世秘籍之争", self.绝世秘籍之争, 0, 200);
        validate_range!("江湖长梦.副本.桃花自古笑春风", self.桃花自古笑春风, 0, 200);
        validate_range!("江湖长梦.副本.战乱襄阳", self.战乱襄阳, 0, 200);
        validate_range!("江湖长梦.副本.天涯浪子", self.天涯浪子, 0, 200);
        validate_range!("江湖长梦.副本.全真古墓意难平", self.全真古墓意难平, 0, 200);
        validate_range!("江湖长梦.副本.南海有岛名侠客", self.南海有岛名侠客, 0, 200);
        Ok(())
    }
}

/// 长梦兑换
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ChangMengExchange {
    /// 兑换上限，0~50
    pub 玄铁令: u32,
    /// 兑换上限，0~50
    pub 淬火结晶: u32,
    /// 兑换上限，0~50
    pub 石中剑: u32,
    /// 兑换上限，0~50
    pub 大型武器符咒: u32,
    /// 兑换上限，0~50
    pub 中型武器符咒: u32,
    /// 兑换上限，0~50
    pub 小型武器符咒: u32,
    /// 兑换上限，0~50
    pub 投掷武器符咒: u32,
}

impl ChangMengExchange {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_range!("江湖长梦.兑换上限.玄铁令", self.玄铁令, 0, 50);
        validate_range!("江湖长梦.兑换上限.淬火结晶", self.淬火结晶, 0, 50);
        validate_range!("江湖长梦.兑换上限.石中剑", self.石中剑, 0, 50);
        validate_range!("江湖长梦.兑换上限.大型武器符咒", self.大型武器符咒, 0, 50);
        validate_range!("江湖长梦.兑换上限.中型武器符咒", self.中型武器符咒, 0, 50);
        validate_range!("江湖长梦.兑换上限.小型武器符咒", self.小型武器符咒, 0, 50);
        validate_range!("江湖长梦.兑换上限.投掷武器符咒", self.投掷武器符咒, 0, 50);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 副本名称映射到执行次数上限，未配置的名称返回 0
    #[test]
    fn test_copy_limit() {
        let copy = ChangMengCopy {
            柒承的忙碌日常: 3,
            倚天屠龙归我心: 4,
            绝世秘籍之争: 5,
            桃花自古笑春风: 6,
            战乱襄阳: 7,
            天涯浪子: 8,
            全真古墓意难平: 9,
            南海有岛名侠客: 10,
        };
        assert_eq!(copy.limit("柒承的忙碌日常"), 3);
        assert_eq!(copy.limit("倚天屠龙归我心"), 4);
        assert_eq!(copy.limit("绝世秘籍之争"), 5);
        assert_eq!(copy.limit("桃花自古笑春风"), 6);
        assert_eq!(copy.limit("战乱襄阳"), 7);
        assert_eq!(copy.limit("天涯浪子"), 8);
        assert_eq!(copy.limit("全真古墓意难平"), 9);
        assert_eq!(copy.limit("南海有岛名侠客"), 10);
        assert_eq!(copy.limit("未知副本"), 0);
    }

    // 兑换上限超上限报错
    #[test]
    fn test_exchange_out_of_range() {
        assert!(
            serde_json::from_str::<ChangMengExchange>(r#"{"玄铁令": 51}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengExchange>(r#"{"淬火结晶": 51}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengExchange>(r#"{"石中剑": 51}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengExchange>(r#"{"大型武器符咒": 51}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengExchange>(r#"{"中型武器符咒": 51}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengExchange>(r#"{"小型武器符咒": 51}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengExchange>(r#"{"投掷武器符咒": 51}"#)
                .unwrap()
                .validate()
                .is_err()
        );
    }

    // 兑换上限边界值合法
    #[test]
    fn test_exchange_range_ok() {
        let json = r#"{"玄铁令": 50, "淬火结晶": 50, "石中剑": 50, "大型武器符咒": 50, "中型武器符咒": 50, "小型武器符咒": 50, "投掷武器符咒": 50}"#;
        let config: ChangMengExchange = serde_json::from_str(json).unwrap();
        config.validate().unwrap();
        assert_eq!(config.玄铁令, 50);
        assert_eq!(config.淬火结晶, 50);
        assert_eq!(config.石中剑, 50);
        assert_eq!(config.大型武器符咒, 50);
        assert_eq!(config.中型武器符咒, 50);
        assert_eq!(config.小型武器符咒, 50);
        assert_eq!(config.投掷武器符咒, 50);
    }

    // 副本执行次数超上限报错
    #[test]
    fn test_copy_out_of_range() {
        assert!(
            serde_json::from_str::<ChangMengCopy>(r#"{"柒承的忙碌日常": 201}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengCopy>(r#"{"倚天屠龙归我心": 201}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengCopy>(r#"{"绝世秘籍之争": 201}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengCopy>(r#"{"桃花自古笑春风": 201}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengCopy>(r#"{"战乱襄阳": 201}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengCopy>(r#"{"天涯浪子": 201}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengCopy>(r#"{"全真古墓意难平": 201}"#)
                .unwrap()
                .validate()
                .is_err()
        );
        assert!(
            serde_json::from_str::<ChangMengCopy>(r#"{"南海有岛名侠客": 201}"#)
                .unwrap()
                .validate()
                .is_err()
        );
    }

    // 副本执行次数边界值合法
    #[test]
    fn test_copy_range_ok() {
        let json = r#"{"柒承的忙碌日常": 200, "绝世秘籍之争": 200}"#;
        let config: ChangMengCopy = serde_json::from_str(json).unwrap();
        config.validate().unwrap();
        assert_eq!(config.柒承的忙碌日常, 200);
        assert_eq!(config.绝世秘籍之争, 200);
    }
}
