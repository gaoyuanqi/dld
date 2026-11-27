//! 帮派商会

use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 帮派商会
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct BangPaiShangHui {
    /// 物品交易
    pub 交易会所: JiaoYiHuiSuo,
    /// 兑换物品
    pub 兑换商店: DuiHuanShangDian,
}

/// 交易会所
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JiaoYiHuiSuo {
    #[serde(rename = "孟婆汤*1")]
    pub 孟婆汤: bool,
    #[serde(rename = "还魂丹*20")]
    pub 还魂丹_20: bool,
    #[serde(rename = "还魂丹*100")]
    pub 还魂丹_100: bool,
    #[serde(rename = "悟性丹*20")]
    pub 悟性丹: bool,
    #[serde(rename = "百炼钢*5")]
    pub 百炼钢_5: bool,
    #[serde(rename = "百炼钢*20")]
    pub 百炼钢_20: bool,
    #[serde(rename = "真体力*5")]
    pub 真体力: bool,
    #[serde(rename = "大体力*5")]
    pub 大体力_5: bool,
    #[serde(rename = "大体力*10")]
    pub 大体力_10: bool,
    #[serde(rename = "小体力*25")]
    pub 小体力: bool,
    #[serde(rename = "挑战书*5")]
    pub 挑战书_5: bool,
    #[serde(rename = "挑战书*10")]
    pub 挑战书_10: bool,
    #[serde(rename = "经验药水*5")]
    pub 经验药水: bool,
    #[serde(rename = "黄金卷轴*5")]
    pub 黄金卷轴: bool,
    #[serde(rename = "神来拳套*5")]
    pub 神来拳套: bool,
    #[serde(rename = "还童卷轴*5")]
    pub 还童卷轴: bool,
    #[serde(rename = "神兵原石*5")]
    pub 神兵原石: bool,
    #[serde(rename = "软猥金丝*5")]
    pub 软猥金丝: bool,
    #[serde(rename = "凤凰羽毛*5")]
    pub 凤凰羽毛: bool,
    #[serde(rename = "潜能果实*5")]
    pub 潜能果实: bool,
    #[serde(rename = "奔流气息*5")]
    pub 奔流气息: bool,
    #[serde(rename = "上古玉髓*5")]
    pub 上古玉髓: bool,
    #[serde(rename = "经验木简*10")]
    pub 经验木简: bool,
    #[serde(rename = "大经验药水*1")]
    pub 大经验药水_1: bool,
    #[serde(rename = "大经验药水*2")]
    pub 大经验药水_2: bool,
    #[serde(rename = "投掷武器符文石*5")]
    pub 投掷武器符文石: bool,
    #[serde(rename = "小型武器符文石*5")]
    pub 小型武器符文石: bool,
    #[serde(rename = "中型武器符文石*5")]
    pub 中型武器符文石: bool,
    #[serde(rename = "大型武器符文石*5")]
    pub 大型武器符文石: bool,
    #[serde(rename = "巅峰之战二等勋章*2")]
    pub 巅峰之战二等勋章: bool,
}

impl JiaoYiHuiSuo {
    /// 返回所有交易物品的（匹配文本片段, 是否开启）列表
    ///
    /// 匹配时拼接中文逗号防止数量误匹配（如 *20 不会误匹配 *2）
    fn items(&self) -> [(&str, bool); 30] {
        [
            ("孟婆汤*1，", self.孟婆汤),
            ("还魂丹*20，", self.还魂丹_20),
            ("还魂丹*100，", self.还魂丹_100),
            ("悟性丹*20，", self.悟性丹),
            ("百炼钢*5，", self.百炼钢_5),
            ("百炼钢*20，", self.百炼钢_20),
            ("真体力*5，", self.真体力),
            ("大体力*5，", self.大体力_5),
            ("大体力*10，", self.大体力_10),
            ("小体力*25，", self.小体力),
            ("挑战书*5，", self.挑战书_5),
            ("挑战书*10，", self.挑战书_10),
            ("经验药水*5，", self.经验药水),
            ("黄金卷轴*5，", self.黄金卷轴),
            ("神来拳套*5，", self.神来拳套),
            ("还童卷轴*5，", self.还童卷轴),
            ("神兵原石*5，", self.神兵原石),
            ("软猥金丝*5，", self.软猥金丝),
            ("凤凰羽毛*5，", self.凤凰羽毛),
            ("潜能果实*5，", self.潜能果实),
            ("奔流气息*5，", self.奔流气息),
            ("上古玉髓*5，", self.上古玉髓),
            ("经验木简*10，", self.经验木简),
            ("大经验药水*1，", self.大经验药水_1),
            ("大经验药水*2，", self.大经验药水_2),
            ("投掷武器符文石*5，", self.投掷武器符文石),
            ("小型武器符文石*5，", self.小型武器符文石),
            ("中型武器符文石*5，", self.中型武器符文石),
            ("大型武器符文石*5，", self.大型武器符文石),
            ("巅峰之战二等勋章*2，", self.巅峰之战二等勋章),
        ]
    }

    /// 判断 tips 是否匹配任一已开启的交易物品
    pub fn is_match(&self, tips: &str) -> bool {
        self.items()
            .iter()
            .any(|(s, enabled)| *enabled && tips.contains(s))
    }
}

/// 兑换商店
///
/// 物品名称列表，上限 10 个，不允许重复
#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct DuiHuanShangDian(Vec<String>);

impl Default for DuiHuanShangDian {
    fn default() -> Self {
        Self(vec!["泯灭·黑炎V碎片".to_string()])
    }
}

impl DuiHuanShangDian {
    /// 判断物品名是否在兑换列表中
    pub fn should_exchange(&self, goods_name: &str) -> bool {
        self.0.iter().any(|s| s == goods_name)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.0.len() > 10 {
            bail!("帮派商会.兑换商店 物品上限 10 个，当前 {} 个", self.0.len());
        }
        let unique: HashSet<_> = self.0.iter().collect();
        if unique.len() != self.0.len() {
            bail!("帮派商会.兑换商店 物品不允许重复");
        }
        for (i, item) in self.0.iter().enumerate() {
            let chars = item.chars().count();
            if chars == 0 || chars > 15 {
                bail!(
                    "帮派商会.兑换商店[{}] 期望 1~15 字符，实际为 {} 字符：\"{item}\"",
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

    /// 构造所有字段为指定值的 JiaoYiHuiSuo
    fn all_items(v: bool) -> JiaoYiHuiSuo {
        JiaoYiHuiSuo {
            孟婆汤: v,
            还魂丹_20: v,
            还魂丹_100: v,
            悟性丹: v,
            百炼钢_5: v,
            百炼钢_20: v,
            真体力: v,
            大体力_5: v,
            大体力_10: v,
            小体力: v,
            挑战书_5: v,
            挑战书_10: v,
            经验药水: v,
            黄金卷轴: v,
            神来拳套: v,
            还童卷轴: v,
            神兵原石: v,
            软猥金丝: v,
            凤凰羽毛: v,
            潜能果实: v,
            奔流气息: v,
            上古玉髓: v,
            经验木简: v,
            大经验药水_1: v,
            大经验药水_2: v,
            投掷武器符文石: v,
            小型武器符文石: v,
            中型武器符文石: v,
            大型武器符文石: v,
            巅峰之战二等勋章: v,
        }
    }

    // items 覆盖全部 30 种交易物品
    #[test]
    fn test_is_match_items_count() {
        let huisuo = JiaoYiHuiSuo::default();
        assert_eq!(huisuo.items().len(), 30);
    }

    // 开启的物品全部能匹配
    #[test]
    fn test_is_match_all_enabled() {
        let huisuo = all_items(true);
        for (s, enabled) in huisuo.items() {
            assert!(enabled, "items() 中 {s:?} 应为 true");
            let tip = format!("教主想要{s}他将返回给你200商会银币");
            assert!(huisuo.is_match(&tip), "开启后应匹配：{s}");
        }
    }

    // 默认关闭的物品全部不匹配
    #[test]
    fn test_is_match_all_disabled() {
        let huisuo = JiaoYiHuiSuo::default();
        for (s, _) in huisuo.items() {
            let tip = format!("教主想要{s}他将返回给你200商会银币");
            assert!(!huisuo.is_match(&tip), "默认关闭不应匹配：{s}");
        }
    }

    // 数量不符不匹配：*20 的用 *2 测，*5 的用 *50 测
    #[test]
    fn test_is_match_quantity_mismatch() {
        let huisuo = all_items(true);
        assert!(!huisuo.is_match("教主想要还魂丹*2，他将返回给你200商会银币"));
        assert!(!huisuo.is_match("教主想要大体力*50，他将返回给你200商会银币"));
        assert!(!huisuo.is_match("教主想要经验木简*1，他将返回给你200商会银币"));
        assert!(!huisuo.is_match("教主想要大经验药水*10，他将返回给你200商会银币"));
        assert!(!huisuo.is_match("教主想要巅峰之战二等勋章*1，他将返回给你200商会银币"));
    }

    // ─── DuiHuanShangDian 测试 ───

    // 默认兑换物品为泯灭·黑炎V碎片
    #[test]
    fn test_dui_huan_shang_dian_default() {
        let d = DuiHuanShangDian::default();
        assert_eq!(d.0, vec!["泯灭·黑炎V碎片"]);
    }

    // 物品数量超上限拒绝
    #[test]
    fn test_dui_huan_shang_dian_too_many() {
        let d = DuiHuanShangDian((0..11).map(|i| format!("物品{i}")).collect());
        assert!(d.validate().is_err());
    }

    // 物品重复拒绝
    #[test]
    fn test_dui_huan_shang_dian_duplicate() {
        let d = DuiHuanShangDian(vec![
            "物品1".to_string(),
            "物品2".to_string(),
            "物品1".to_string(),
        ]);
        assert!(d.validate().is_err());
    }

    // 合法列表通过
    #[test]
    fn test_dui_huan_shang_dian_valid() {
        let d = DuiHuanShangDian(vec!["物品1".to_string(), "物品2".to_string()]);
        assert!(d.validate().is_ok());
    }

    // 物品名空字符串拒绝
    #[test]
    fn test_dui_huan_shang_dian_empty_name() {
        let d = DuiHuanShangDian(vec!["".to_string()]);
        assert!(d.validate().is_err());
    }

    // 物品名超 15 字符拒绝
    #[test]
    fn test_dui_huan_shang_dian_name_too_long() {
        let d = DuiHuanShangDian(vec!["物".repeat(16)]);
        assert!(d.validate().is_err());
    }

    // 物品名 15 字符合法
    #[test]
    fn test_dui_huan_shang_dian_name_max_chars() {
        let d = DuiHuanShangDian(vec!["物".repeat(15)]);
        assert!(d.validate().is_ok());
    }

    // 按物品名判断是否兑换
    #[test]
    fn test_should_exchange() {
        let d = DuiHuanShangDian(vec!["a".to_string(), "b".to_string()]);
        assert!(d.should_exchange("a"));
        assert!(!d.should_exchange("c"));
    }
}
