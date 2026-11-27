//! 开学季账号配置：疯狂许愿奖励选择

use serde::{Deserialize, Serialize};

/// 开学季
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct KaiXueJi {
    pub 选择: FengKuangXuYuan,
}

/// 疯狂许愿具体奖励
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum FengKuangXuYuan {
    #[serde(rename = "神兵")]
    神兵,
    #[serde(rename = "神铠")]
    神铠,
    #[serde(rename = "神羽")]
    神羽,
    #[serde(rename = "神兽")]
    神兽,
    #[serde(rename = "神饰")]
    神饰,
    #[serde(rename = "神履")]
    神履,
    #[serde(rename = "坚韧不拔")]
    坚韧不拔,
    #[serde(rename = "嗜血如命")]
    嗜血如命,
    #[serde(rename = "坚定不移")]
    #[default]
    坚定不移,
    #[serde(rename = "生存本能")]
    生存本能,
    #[serde(rename = "横扫千军")]
    横扫千军,
    #[serde(rename = "三魂之力")]
    三魂之力,
    #[serde(rename = "四魂天功")]
    四魂天功,
    #[serde(rename = "炙血战魂")]
    炙血战魂,
    #[serde(rename = "百战之躯")]
    百战之躯,
    #[serde(rename = "攻无不克")]
    攻无不克,
    #[serde(rename = "投掷武器专精")]
    投掷武器专精,
    #[serde(rename = "小型武器专精")]
    小型武器专精,
    #[serde(rename = "中型武器专精")]
    中型武器专精,
    #[serde(rename = "大型武器专精")]
    大型武器专精,
    #[serde(rename = "强化")]
    强化,
}

impl FengKuangXuYuan {
    /// 具体奖励
    pub(crate) fn item_name(&self) -> &str {
        match self {
            Self::神兵 => "神兵",
            Self::神铠 => "神铠",
            Self::神羽 => "神羽",
            Self::神兽 => "神兽",
            Self::神饰 => "神饰",
            Self::神履 => "神履",
            Self::坚韧不拔 => "坚韧不拔",
            Self::嗜血如命 => "嗜血如命",
            Self::坚定不移 => "坚定不移",
            Self::生存本能 => "生存本能",
            Self::横扫千军 => "横扫千军",
            Self::三魂之力 => "三魂之力",
            Self::四魂天功 => "四魂天功",
            Self::炙血战魂 => "炙血战魂",
            Self::百战之躯 => "百战之躯",
            Self::攻无不克 => "攻无不克",
            Self::投掷武器专精 => "投掷武器专精",
            Self::小型武器专精 => "小型武器专精",
            Self::中型武器专精 => "中型武器专精",
            Self::大型武器专精 => "大型武器专精",
            Self::强化 => "强化",
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
            FengKuangXuYuan::神兵,
            FengKuangXuYuan::神铠,
            FengKuangXuYuan::神羽,
            FengKuangXuYuan::神兽,
            FengKuangXuYuan::神饰,
            FengKuangXuYuan::神履,
            FengKuangXuYuan::坚韧不拔,
            FengKuangXuYuan::嗜血如命,
            FengKuangXuYuan::坚定不移,
            FengKuangXuYuan::生存本能,
            FengKuangXuYuan::横扫千军,
            FengKuangXuYuan::三魂之力,
            FengKuangXuYuan::四魂天功,
            FengKuangXuYuan::炙血战魂,
            FengKuangXuYuan::百战之躯,
            FengKuangXuYuan::攻无不克,
            FengKuangXuYuan::投掷武器专精,
            FengKuangXuYuan::小型武器专精,
            FengKuangXuYuan::中型武器专精,
            FengKuangXuYuan::大型武器专精,
            FengKuangXuYuan::强化,
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
