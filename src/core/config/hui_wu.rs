//! 会武

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 会武
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct HuiWu {
    /// 成功助威后兑换真黄金卷轴数量，0~100
    pub 兑换真黄金卷轴数量: u32,
}

impl HuiWu {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_range!("会武.兑换真黄金卷轴数量", self.兑换真黄金卷轴数量, 0, 100);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 卷轴数量超上限报错
    #[test]
    fn test_out_of_range() {
        let config: HuiWu = serde_json::from_str(r#"{"兑换真黄金卷轴数量": 101}"#).unwrap();
        assert!(config.validate().is_err());
    }

    // 卷轴数量边界值合法
    #[test]
    fn test_range_ok() {
        let config: HuiWu = serde_json::from_str(r#"{"兑换真黄金卷轴数量": 100}"#).unwrap();
        config.validate().unwrap();
        assert_eq!(config.兑换真黄金卷轴数量, 100);
    }
}
