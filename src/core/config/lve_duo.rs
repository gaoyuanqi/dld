//! 掠夺账号配置：筛选对手战力门槛与递增步长

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 掠夺
///
/// | 目标战力 | 战力增量 | 行为 |
/// |---|---|------|
/// | >0 | >0 | 从指定门槛开始，找不到逐步升高 |
/// | >0 | =0 | 只打门槛以下的，找不到就停 |
/// | =0 | >0 | 从 0 开始自动递增 |
/// | =0 | =0 | 跳过掠夺 |
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LueDuo {
    /// 筛选对手的起始战力阈值
    pub 目标战力: u32,
    /// 每轮递增步长，设为 0 则不递增
    pub 战力增量: u32,
}

impl Default for LueDuo {
    fn default() -> Self {
        Self {
            目标战力: 1000,
            战力增量: 0,
        }
    }
}

impl LueDuo {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_range!("掠夺.目标战力", self.目标战力, 0, 99999);
        validate_range!("掠夺.战力增量", self.战力增量, 0, 9999);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> LueDuo {
        serde_json::from_str(json).unwrap()
    }

    // 超上限报错
    #[test]
    fn test_out_of_range() {
        assert!(parse(r#"{"目标战力": 100000}"#).validate().is_err());
        assert!(parse(r#"{"战力增量": 10000}"#).validate().is_err());
    }

    // 边界值合法
    #[test]
    fn test_range_ok() {
        let config = parse(r#"{"目标战力": 99999, "战力增量": 9999}"#);
        config.validate().unwrap();
        assert_eq!(config.目标战力, 99999);
        assert_eq!(config.战力增量, 9999);
    }
}
