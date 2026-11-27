//! 历练

use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 历练
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LiLian {
    /// BOSS 乐斗顺序，必须包含全部 9 个且不重复
    pub 乐斗顺序: Vec<LiLianBoss>,
}

impl Default for LiLian {
    fn default() -> Self {
        Self {
            乐斗顺序: vec![
                LiLianBoss::XiongShi,
                LiLianBoss::XiaBingTouMu,
                LiLianBoss::YeChaYuanShuai,
                LiLianBoss::PiLiTouLing,
                LiLianBoss::SongJiang,
                LiLianBoss::DaPeng,
                LiLianBoss::MaDaWang,
                LiLianBoss::ShiXueGuiWang,
                LiLianBoss::XiangXian,
            ],
        }
    }
}

impl LiLian {
    pub(crate) fn validate(&self) -> Result<()> {
        let bosses = &self.乐斗顺序;
        if bosses.len() != 9 {
            bail!(
                "历练.乐斗顺序 必须包含全部 9 个 BOSS，当前 {} 个",
                bosses.len()
            );
        }
        let unique: HashSet<_> = bosses.iter().collect();
        if unique.len() != bosses.len() {
            bail!("历练.乐斗顺序 不允许重复");
        }
        Ok(())
    }
}

/// 历练 BOSS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum LiLianBoss {
    #[serde(rename = "凶尸-令狐冲")]
    XiongShi,
    #[serde(rename = "虾兵头目-丁春秋")]
    XiaBingTouMu,
    #[serde(rename = "夜叉元帅-丘处机")]
    YeChaYuanShuai,
    #[serde(rename = "霹雳头领-小龙女")]
    PiLiTouLing,
    #[serde(rename = "宋姜-韦小宝")]
    SongJiang,
    #[serde(rename = "大鹏-扫地僧")]
    DaPeng,
    #[serde(rename = "马大王-鹤笔翁")]
    MaDaWang,
    #[serde(rename = "嗜血鬼王-韦一笑")]
    ShiXueGuiWang,
    #[serde(rename = "象仙-赵敏")]
    XiangXian,
}

impl LiLianBoss {
    /// 所属场景 MapID
    pub fn mapid(self) -> &'static str {
        match self {
            Self::XiongShi => "2",
            Self::XiaBingTouMu => "3",
            Self::YeChaYuanShuai => "4",
            Self::PiLiTouLing => "5",
            Self::SongJiang => "6",
            Self::DaPeng => "7",
            Self::MaDaWang => "8",
            Self::ShiXueGuiWang => "9",
            Self::XiangXian => "10",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 构造乐斗顺序 JSON 并解析
    fn parse(bosses: &[&str]) -> LiLian {
        let list = bosses
            .iter()
            .map(|b| format!("\"{b}\""))
            .collect::<Vec<_>>()
            .join(",");
        serde_json::from_str(&format!("{{\"乐斗顺序\": [{list}]}}")).unwrap()
    }

    // BOSS 顺序数量不足校验
    #[test]
    fn test_too_few() {
        assert!(parse(&["凶尸-令狐冲"]).validate().is_err());
    }

    // BOSS 顺序数量过多校验
    #[test]
    fn test_too_many() {
        let bosses = [
            "凶尸-令狐冲",
            "虾兵头目-丁春秋",
            "夜叉元帅-丘处机",
            "霹雳头领-小龙女",
            "宋姜-韦小宝",
            "大鹏-扫地僧",
            "马大王-鹤笔翁",
            "嗜血鬼王-韦一笑",
            "象仙-赵敏",
            "象仙-赵敏",
        ];
        assert!(parse(&bosses).validate().is_err());
    }

    // BOSS 顺序重复校验
    #[test]
    fn test_duplicate() {
        let bosses = [
            "凶尸-令狐冲",
            "虾兵头目-丁春秋",
            "夜叉元帅-丘处机",
            "霹雳头领-小龙女",
            "宋姜-韦小宝",
            "大鹏-扫地僧",
            "马大王-鹤笔翁",
            "嗜血鬼王-韦一笑",
            "嗜血鬼王-韦一笑",
        ];
        assert!(parse(&bosses).validate().is_err());
    }

    // BOSS 顺序合法自定义配置
    #[test]
    fn test_custom() {
        let bosses = [
            "象仙-赵敏",
            "凶尸-令狐冲",
            "虾兵头目-丁春秋",
            "夜叉元帅-丘处机",
            "霹雳头领-小龙女",
            "宋姜-韦小宝",
            "大鹏-扫地僧",
            "马大王-鹤笔翁",
            "嗜血鬼王-韦一笑",
        ];
        let config = parse(&bosses);
        config.validate().unwrap();
        assert_eq!(config.乐斗顺序[0], LiLianBoss::XiangXian);
        assert_eq!(config.乐斗顺序[1], LiLianBoss::XiongShi);
        assert_eq!(config.乐斗顺序.len(), 9);
    }

    // BOSS 顺序无效枚举值应报错
    #[test]
    fn test_invalid_boss() {
        let bosses = [
            "凶尸-令狐冲",
            "虾兵头目-丁春秋",
            "夜叉元帅-丘处机",
            "霹雳头领-小龙女",
            "宋姜-韦小宝",
            "大鹏-扫地僧",
            "马大王-鹤笔翁",
            "嗜血鬼王-韦一笑",
            "不存在-BOSS",
        ];
        assert!(
            serde_json::from_str::<LiLian>(&format!(
                "{{\"乐斗顺序\": [{}]}}",
                bosses
                    .iter()
                    .map(|b| format!("\"{b}\""))
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .is_err()
        );
    }
}
