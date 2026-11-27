//! 门派邀请赛账号配置：兑换上限

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 门派邀请赛
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MenPaiYaoQingSai {
    pub 兑换: MenPaiExchange,
}

/// 门派邀请赛兑换
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MenPaiExchange {
    /// 兑换上限，0~20
    pub 炼气石: u32,
    /// 兑换上限，0~20
    pub 门派强化书: u32,
}

impl MenPaiExchange {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_range!("门派邀请赛.兑换.炼气石", self.炼气石, 0, 20);
        validate_range!("门派邀请赛.兑换.门派强化书", self.门派强化书, 0, 20);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> MenPaiExchange {
        serde_json::from_str(json).unwrap()
    }

    // 兑换上限超限报错
    #[test]
    fn test_out_of_range() {
        assert!(parse(r#"{"炼气石": 21}"#).validate().is_err());
        assert!(parse(r#"{"门派强化书": 21}"#).validate().is_err());
    }

    // 兑换上限边界值合法
    #[test]
    fn test_range_ok() {
        let config = parse(r#"{"炼气石": 20, "门派强化书": 20}"#);
        config.validate().unwrap();
        assert_eq!(config.炼气石, 20);
        assert_eq!(config.门派强化书, 20);
    }
}
