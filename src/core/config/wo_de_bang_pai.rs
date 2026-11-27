//! 我的帮派

use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 我的帮派
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WoDeBangPai {
    pub 报名: bool,
    pub 供奉: Vec<String>,
}

impl Default for WoDeBangPai {
    fn default() -> Self {
        Self {
            报名: false,
            供奉: vec!["还魂丹".to_string()],
        }
    }
}

impl WoDeBangPai {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.供奉.len() > 40 {
            bail!("我的帮派.供奉 期望 0~40 个，当前 {} 个", self.供奉.len());
        }
        let unique: HashSet<_> = self.供奉.iter().collect();
        if unique.len() != self.供奉.len() {
            bail!("我的帮派.供奉 物品不允许重复");
        }
        for (i, item) in self.供奉.iter().enumerate() {
            let chars = item.chars().count();
            if chars == 0 || chars > 15 {
                bail!(
                    "我的帮派.供奉[{}] 期望 1~15 字符，实际为 {} 字符：\"{item}\"",
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

    // 供奉数量超限
    #[test]
    fn test_too_many_offerings() {
        let config = WoDeBangPai {
            报名: false,
            供奉: (0..41).map(|i| format!("物品{i}")).collect(),
        };
        assert!(config.validate().is_err());
    }

    // 供奉元素超长
    #[test]
    fn test_item_too_long() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["a".repeat(16)],
        };
        assert!(config.validate().is_err());
    }

    // 供奉空字符串拒绝
    #[test]
    fn test_empty_offering() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["".to_string()],
        };
        assert!(config.validate().is_err());
    }

    // 供奉重复拒绝
    #[test]
    fn test_duplicate_offering() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["还魂丹".to_string(), "还魂丹".to_string()],
        };
        assert!(config.validate().is_err());
    }

    // 中文按字符数计：11 个汉字（33 字节）合法
    #[test]
    fn test_chinese_chars_valid() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["供".repeat(11)],
        };
        assert!(config.validate().is_ok());
    }

    // 超过 15 个字符拒绝
    #[test]
    fn test_too_many_chars() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["供".repeat(16)],
        };
        assert!(config.validate().is_err());
    }

    // 合法供奉
    #[test]
    fn test_valid() {
        let config = WoDeBangPai {
            报名: false,
            供奉: vec!["还魂丹".to_string(), "经验药水".to_string()],
        };
        assert!(config.validate().is_ok());
    }
}
