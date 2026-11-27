//! 登录商店

use serde::{Deserialize, Serialize};

/// 登录商店
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DengLuShangDian {
    pub 兑换: DengLuShangDianExchange,
}

/// 登录商店兑换材料
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum DengLuShangDianExchange {
    #[serde(rename = "黄金卷轴")]
    黄金卷轴,
    #[serde(rename = "孙子兵法")]
    孙子兵法,
    #[serde(rename = "武穆遗书")]
    武穆遗书,
    #[serde(rename = "神秘精华")]
    神秘精华,
    #[serde(rename = "神兵原石")]
    #[default]
    神兵原石,
    #[serde(rename = "软猥金丝")]
    软猥金丝,
    #[serde(rename = "凤凰羽毛")]
    凤凰羽毛,
    #[serde(rename = "潜能果实")]
    潜能果实,
    #[serde(rename = "上古玉髓")]
    上古玉髓,
    #[serde(rename = "奔流气息")]
    奔流气息,
}

impl DengLuShangDianExchange {
    /// 物品名称，用于匹配接口返回的兑换物品列表
    pub(crate) fn item_name(&self) -> &str {
        match self {
            Self::黄金卷轴 => "黄金卷轴",
            Self::孙子兵法 => "孙子兵法",
            Self::武穆遗书 => "武穆遗书",
            Self::神秘精华 => "神秘精华",
            Self::神兵原石 => "神兵原石",
            Self::软猥金丝 => "软猥金丝",
            Self::凤凰羽毛 => "凤凰羽毛",
            Self::潜能果实 => "潜能果实",
            Self::上古玉髓 => "上古玉髓",
            Self::奔流气息 => "奔流气息",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_name() {
        // item_name 与 serde rename 须保持一致，序列化值作为对照
        let variants = [
            DengLuShangDianExchange::黄金卷轴,
            DengLuShangDianExchange::孙子兵法,
            DengLuShangDianExchange::武穆遗书,
            DengLuShangDianExchange::神秘精华,
            DengLuShangDianExchange::神兵原石,
            DengLuShangDianExchange::软猥金丝,
            DengLuShangDianExchange::凤凰羽毛,
            DengLuShangDianExchange::潜能果实,
            DengLuShangDianExchange::上古玉髓,
            DengLuShangDianExchange::奔流气息,
        ];
        for v in variants {
            let name = serde_json::to_value(&v)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            assert_eq!(v.item_name(), name);
        }
    }
}
