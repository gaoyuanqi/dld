//! 吉利兑

use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 吉利兑
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct JiLiDui {
    pub 兑换优先级: Vec<String>,
}

impl Default for JiLiDui {
    fn default() -> Self {
        Self {
            兑换优先级: vec![
                "V级万能碎片".to_string(),
                "熔炼乌金".to_string(),
                "舆图".to_string(),
                "玄铁令".to_string(),
                "挑战书".to_string(),
            ],
        }
    }
}

impl JiLiDui {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.兑换优先级.len() > 12 {
            bail!(
                "吉利兑.兑换优先级 物品上限 12 个，当前 {} 个",
                self.兑换优先级.len()
            );
        }
        let unique: HashSet<_> = self.兑换优先级.iter().collect();
        if unique.len() != self.兑换优先级.len() {
            bail!("吉利兑.兑换优先级 物品不允许重复");
        }
        for (i, item) in self.兑换优先级.iter().enumerate() {
            let chars = item.chars().count();
            if chars == 0 || chars > 15 {
                bail!(
                    "吉利兑.兑换优先级[{}] 期望 1~15 字符，实际为 {} 字符：\"{item}\"",
                    i,
                    chars
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> JiLiDui {
        serde_json::from_str(json).unwrap()
    }

    // 默认兑换优先级顺序正确
    #[test]
    fn test_default_order() {
        let config = JiLiDui::default();
        let want = ["V级万能碎片", "熔炼乌金", "舆图", "玄铁令", "挑战书"];
        assert_eq!(config.兑换优先级.len(), want.len());
        for (got, want) in config.兑换优先级.iter().zip(want.iter()) {
            assert_eq!(got, want);
        }
    }

    // 配置保留用户值
    #[test]
    fn test_load_valid() {
        let config = parse(r#"{"兑换优先级": ["挑战书", "玄铁令"]}"#);
        config.validate().unwrap();
        assert_eq!(config.兑换优先级, ["挑战书", "玄铁令"]);
    }

    // 物品名称重复报错
    #[test]
    fn test_duplicate_errors() {
        assert!(
            parse(r#"{"兑换优先级": ["挑战书", "挑战书"]}"#)
                .validate()
                .is_err()
        );
    }

    // 物品数量超上限报错
    #[test]
    fn test_too_many_errors() {
        let items: Vec<String> = (0..13).map(|i| format!("物品{i}")).collect();
        let json = serde_json::json!({ "兑换优先级": items }).to_string();
        assert!(parse(&json).validate().is_err());
    }

    // 空名称报错
    #[test]
    fn test_empty_name_errors() {
        assert!(parse(r#"{"兑换优先级": [""]}"#).validate().is_err());
    }
}
