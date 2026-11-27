//! 侠士客栈账号配置：黑市商人奇遇交换开关

use serde::{Deserialize, Serialize};

/// 侠士客栈
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct XiaShiKeZhan {
    pub 黑市商人: HeiShiShangRen,
}

impl XiaShiKeZhan {
    /// 根据奇遇 advId 判断是否开启交换
    pub fn is_enabled(&self, adv_id: &str) -> bool {
        self.黑市商人.is_enabled(adv_id)
    }
}

/// 黑市商人
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct HeiShiShangRen {
    #[serde(rename = "挑战书*1换取玄铁令*1")]
    pub ex_1006: bool,
    #[serde(rename = "黄金卷轴*2换取斗灵石空*5")]
    pub ex_1010: bool,
    #[serde(rename = "门派强化书*2换取斗灵石空*5")]
    pub ex_1011: bool,
    #[serde(rename = "斗灵石空*66换取V级万能碎片*1")]
    pub ex_1012: bool,
    #[serde(rename = "黄金卷轴*3换取斗神符*1")]
    pub ex_1013: bool,
    #[serde(rename = "无字天书*1换取易经八卦*1")]
    pub ex_1014: bool,
}

impl HeiShiShangRen {
    /// 根据奇遇 advId 判断是否开启交换
    pub fn is_enabled(&self, adv_id: &str) -> bool {
        match adv_id {
            "1006" => self.ex_1006,
            "1010" => self.ex_1010,
            "1011" => self.ex_1011,
            "1012" => self.ex_1012,
            "1013" => self.ex_1013,
            "1014" => self.ex_1014,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_enabled_known_adv_ids() {
        let kezhan = XiaShiKeZhan {
            黑市商人: HeiShiShangRen {
                ex_1006: true,
                ex_1010: false,
                ex_1011: false,
                ex_1012: true,
                ex_1013: false,
                ex_1014: true,
            },
        };
        assert!(kezhan.is_enabled("1006"));
        assert!(!kezhan.is_enabled("1010"));
        assert!(!kezhan.is_enabled("1011"));
        assert!(kezhan.is_enabled("1012"));
        assert!(!kezhan.is_enabled("1013"));
        assert!(kezhan.is_enabled("1014"));
    }

    #[test]
    fn test_is_enabled_unknown_adv_id() {
        let kezhan = XiaShiKeZhan::default();
        assert!(!kezhan.is_enabled("9999"));
        assert!(!kezhan.is_enabled(""));
        assert!(!kezhan.is_enabled("1007"));
    }
}
