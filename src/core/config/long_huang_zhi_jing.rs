//! 龙凰之境

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 龙凰之境
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LongHuangZhiJing {
    pub 兑换上限: LongHuangYunJi,
}

/// 龙凰云集
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LongHuangYunJi {
    /// 兑换上限，0~100
    pub 凰髓: u32,
    /// 兑换上限，0~16
    pub 凰火: u32,
    /// 兑换上限，0~100
    pub 龙玉: u32,
    /// 兑换上限，0~40
    pub 论武券: u32,
}

impl LongHuangYunJi {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_range!("龙凰之境.兑换上限.凰髓", self.凰髓, 0, 100);
        validate_range!("龙凰之境.兑换上限.凰火", self.凰火, 0, 16);
        validate_range!("龙凰之境.兑换上限.龙玉", self.龙玉, 0, 100);
        validate_range!("龙凰之境.兑换上限.论武券", self.论武券, 0, 40);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> LongHuangYunJi {
        serde_json::from_str(json).unwrap()
    }

    // 兑换上限超上限报错
    #[test]
    fn test_out_of_range() {
        assert!(parse(r#"{"凰髓": 101}"#).validate().is_err());
        assert!(parse(r#"{"凰火": 17}"#).validate().is_err());
        assert!(parse(r#"{"龙玉": 101}"#).validate().is_err());
        assert!(parse(r#"{"论武券": 41}"#).validate().is_err());
    }

    // 兑换上限边界值合法
    #[test]
    fn test_range_ok() {
        let config = parse(r#"{"凰髓": 100, "凰火": 16, "龙玉": 100, "论武券": 40}"#);
        config.validate().unwrap();
        assert_eq!(config.凰髓, 100);
        assert_eq!(config.凰火, 16);
        assert_eq!(config.龙玉, 100);
        assert_eq!(config.论武券, 40);
    }
}
