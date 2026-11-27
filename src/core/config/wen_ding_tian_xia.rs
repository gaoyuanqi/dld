//! 问鼎天下账号配置：攻占区域、付费攻占、碎片兑换开关

use serde::{Deserialize, Serialize};

/// 问鼎天下
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct WenDingTianXia {
    /// 攻占资源点区域
    pub 攻占区域: Region,
    /// 付费两次
    pub 付费攻占: bool,
    /// 是否每天兑换碎片
    pub 兑换碎片: bool,
}

/// 攻占区域
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Region {
    #[serde(rename = "东海")]
    #[default]
    东海,
    #[serde(rename = "南荒")]
    南荒,
    #[serde(rename = "西泽")]
    西泽,
    #[serde(rename = "北寒")]
    北寒,
}

impl Region {
    /// 转换为接口参数值
    pub(crate) fn api_value(&self) -> &str {
        match self {
            Self::东海 => "1",
            Self::南荒 => "2",
            Self::西泽 => "3",
            Self::北寒 => "4",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // API 参数映射正确，serde rename 与变体名一致（序列化值作为对照）
    #[test]
    fn test_region_api_value_and_rename() {
        assert_eq!(Region::东海.api_value(), "1");
        assert_eq!(Region::南荒.api_value(), "2");
        assert_eq!(Region::西泽.api_value(), "3");
        assert_eq!(Region::北寒.api_value(), "4");
        let variants = [Region::东海, Region::南荒, Region::西泽, Region::北寒];
        let names = ["东海", "南荒", "西泽", "北寒"];
        for (v, name) in variants.iter().zip(names.iter()) {
            let got = serde_json::to_value(v)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            assert_eq!(&got, name);
        }
    }

    // 兑换碎片开关读取，缺失时用默认值补齐
    #[test]
    fn test_exchange_parse() {
        let config: WenDingTianXia = serde_json::from_str(r#"{"兑换碎片": true}"#).unwrap();
        assert!(config.兑换碎片);
        let config: WenDingTianXia = serde_json::from_str(r#"{}"#).unwrap();
        assert!(!config.兑换碎片);
    }
}
