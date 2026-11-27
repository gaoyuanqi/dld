//! 梦想之旅账号配置：周四梦幻机票消耗上限

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 梦想之旅
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MengXiangZhiLv {
    /// 周四消耗梦幻机票数量， 0~9
    pub 最多消耗梦幻机票数量: u32,
}

impl Default for MengXiangZhiLv {
    fn default() -> Self {
        Self {
            最多消耗梦幻机票数量: 3,
        }
    }
}

impl MengXiangZhiLv {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_range!(
            "梦想之旅.最多消耗梦幻机票数量",
            self.最多消耗梦幻机票数量,
            0,
            9
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 机票数超上限报错
    #[test]
    fn test_out_of_range() {
        let config: MengXiangZhiLv =
            serde_json::from_str(r#"{"最多消耗梦幻机票数量": 10}"#).unwrap();
        assert!(config.validate().is_err());
    }

    // 机票数边界值合法
    #[test]
    fn test_range_ok() {
        let config: MengXiangZhiLv =
            serde_json::from_str(r#"{"最多消耗梦幻机票数量": 9}"#).unwrap();
        config.validate().unwrap();
        assert_eq!(config.最多消耗梦幻机票数量, 9);
    }
}
