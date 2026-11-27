//! 背包

use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 背包
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct BeiBao {
    /// 使用物品名称
    pub 使用: Vec<String>,
}

impl Default for BeiBao {
    fn default() -> Self {
        Self {
            使用: vec![
                "周年福袋".to_string(),
                "福运红包".to_string(),
                "财运红包".to_string(),
                "阅历卷宗".to_string(),
                "贡献小笼包".to_string(),
                "贡献叉烧包".to_string(),
                "阅历羊皮卷".to_string(),
                "装酒肉的食盒".to_string(),
            ],
        }
    }
}

impl BeiBao {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.使用.len() > 50 {
            bail!("背包.使用 物品上限 50 个，当前 {} 个", self.使用.len());
        }
        let unique: HashSet<_> = self.使用.iter().collect();
        if unique.len() != self.使用.len() {
            bail!("背包.使用 物品不允许重复");
        }
        for (i, item) in self.使用.iter().enumerate() {
            let chars = item.chars().count();
            if chars == 0 || chars > 15 {
                bail!(
                    "背包.使用[{}] 期望 1~15 字符，实际为 {} 字符：\"{item}\"",
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

    fn parse_items(items: Vec<String>) -> BeiBao {
        let json = format!(r#"{{"使用": {}}}"#, serde_json::to_string(&items).unwrap());
        serde_json::from_str(&json).unwrap()
    }

    // 使用物品数量超上限报错
    #[test]
    fn test_out_of_range() {
        let items: Vec<String> = (0..51).map(|i| format!("物品{i}")).collect();
        assert!(parse_items(items).validate().is_err());
    }

    // 使用物品数量边界值合法
    #[test]
    fn test_range_ok() {
        let items: Vec<String> = (0..50).map(|i| format!("物品{i}")).collect();
        let config = parse_items(items);
        config.validate().unwrap();
        assert_eq!(config.使用.len(), 50);
    }

    // 使用物品重复报错
    #[test]
    fn test_duplicate() {
        assert!(
            parse_items(vec!["金创药".to_string(), "金创药".to_string()])
                .validate()
                .is_err()
        );
    }

    // 使用物品元素长度非法报错（空字符串 / 16 字符）
    #[test]
    fn test_item_len_out_of_range() {
        assert!(parse_items(vec!["".to_string()]).validate().is_err());
        assert!(
            parse_items(vec!["一二三四五六七八九十一二三四五六".to_string()])
                .validate()
                .is_err()
        );
    }

    // 使用物品元素长度边界值合法（15 字符）
    #[test]
    fn test_item_len_range_ok() {
        let config = parse_items(vec!["一二三四五六七八九十一二三四五".to_string()]);
        config.validate().unwrap();
        assert_eq!(config.使用[0], "一二三四五六七八九十一二三四五");
    }
}
