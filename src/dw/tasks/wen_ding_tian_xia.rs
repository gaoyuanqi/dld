//! 问鼎天下
//!
//! 领取奖励、攻占、助威、积分兑换
//!
//! 攻占：
//! 每天总次数 5（3 次免费 + 2 次付费），left_fight_times 不区分免费/付费
//! 积分最大化：1 级失败 30 分，3 级空资源点成功 150 分
//! 策略：1 级打可用次数-1 次，3 级空资源点打 1 次，成功占领 1 级则放弃
//! 付费攻占用全部次数；免费攻占至多 3 次（扣除固定 2 次付费次数）
//!
//! 助威：傲龙づ族℃
//!
//! 积分兑换：神魔录古阵篇品质未封顶宝物按缺口每天兑换碎片

use std::cmp;
use std::time::Duration;

use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "问鼎天下";

#[derive(Default, Deserialize)]
struct SelfTerritory {
    #[serde(default)]
    canoncial_id: String, // 资源点id
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        duration: String,
        #[serde(default)]
        tb: Tb, // 资源点争夺
        #[serde(default)]
        rb: Cheer, // 区域淘汰赛
        #[serde(default)]
        cb: Cheer, // 冠军排名赛
    }

    #[derive(Default, Deserialize)]
    struct Tb {
        #[serde(default)]
        release_reward: String, // 资源点奖励
        #[serde(default)]
        self_territory: SelfTerritory,
    }

    #[derive(Default, Deserialize)]
    struct Cheer {
        #[serde(default)]
        can_cheer: String, // 是否可助威
        #[serde(default)]
        cheered_faction_id: String, // 助威帮派id
    }

    let data: Query = match d.get("cmd=tbattle").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    if data.duration == "1" {
        if data.rb.can_cheer == "1" && data.rb.cheered_faction_id == "0" {
            淘汰赛助威(d).await;
            积分兑换(d).await;
        }
        return;
    }
    if data.duration == "2" {
        if data.cb.can_cheer == "1" && data.cb.cheered_faction_id == "0" {
            排名赛助威(d).await;
            积分兑换(d).await;
        }
        return;
    }

    // 非资源点争夺时间
    if data.duration != "0" {
        return;
    }

    if !data.tb.release_reward.is_empty() {
        领取(d).await;
        领取奖励(d).await;
    }

    if data.tb.self_territory.canoncial_id == "0" {
        资源点争夺(d).await;
    }
}

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

async fn 领取奖励(d: &DaLeDou) {
    // 领取奖励
    let data: Response = match d.get("cmd=tbattle&op=drawreward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取(d: &DaLeDou) {
    // 领取
    let data: Response = match d.get("cmd=tbattle&op=drawreleasereward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 淘汰赛助威(d: &DaLeDou) {
    // 助威傲龙づ族℃
    let data: Response = match d
        .get("cmd=tbattle&op=cheerregionbattle&faction=96690")
        .await
    {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 排名赛助威(d: &DaLeDou) {
    // 助威傲龙づ族℃
    let data: Response = match d
        .get("cmd=tbattle&op=cheerchampionbattle&faction=96690")
        .await
    {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 资源点争夺(d: &DaLeDou) {
    let region_id = d.config().问鼎天下.攻占区域.api_value();
    let is_paid = d.config().问鼎天下.付费攻占;

    let Some(data) = region(d, region_id).await else {
        return;
    };

    if data.left_fight_times == "0" {
        return;
    }

    if !is_paid && (data.left_fight_times == "1" || data.left_fight_times == "2") {
        return;
    }

    let left_times: u32 = match data.left_fight_times.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 left_fight_times 字段失败：{e}"));
            return;
        }
    };

    let Some(level_1_count) = calc_attack_count(left_times, is_paid) else {
        return;
    };
    if level_1_count > 0 {
        let Some(level_1_data) = data
            .level_1_territories
            .iter()
            .find(|t| t.owner_combat_points != "0.0")
        else {
            return;
        };

        for _ in 0..level_1_count {
            // 成功占领则放弃
            if 攻占(d, &level_1_data.canoncial_id, region_id).await {
                放弃(d).await;
            }
        }
    }

    let Some(level_3_data) = data.level_3_territories.last() else {
        return;
    };

    // 攻占3级最后一个空资源点
    if level_3_data.owner_combat_points == "0.0" {
        攻占(d, &level_3_data.canoncial_id, region_id).await;
        积分兑换(d).await;
    }
}

#[derive(Deserialize)]
struct Region {
    result: String,
    msg: String,
    left_fight_times: String, // 剩余抢占次数
    self_territory: SelfTerritory,
    level_1_territories: Vec<Territory>, // 1级资源点列表
    level_3_territories: Vec<Territory>, // 3级资源点列表
}

#[derive(Deserialize)]
struct Territory {
    canoncial_id: String,        // 资源点id
    owner_combat_points: String, // 领主战力
}

async fn region(d: &DaLeDou, region: &str) -> Option<Region> {
    // 攻占区域
    let cmd = format!("cmd=tbattle&op=showregion&region={region}");
    let data: Region = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    if data.left_fight_times == "0" {
        return None;
    }

    // 已有占领
    if data.self_territory.canoncial_id != "0" {
        return None;
    }

    Some(data)
}

/// 计算攻占计划：返回 1 级资源点攻占次数（3 级固定最后 1 次）
/// 付费攻占使用全部次数，免费攻占至多 3 次（扣除固定 2 次付费次数）
/// 返回 None 表示无可用攻占次数
fn calc_attack_count(left_times: u32, is_paid: bool) -> Option<u32> {
    let total = if is_paid {
        left_times
    } else {
        cmp::min(3, left_times.saturating_sub(2))
    };
    if total == 0 {
        return None;
    }
    Some(total - 1)
}

async fn 攻占(d: &DaLeDou, id: &str, region_id: &str) -> bool {
    // 攻占
    let cmd = format!("cmd=tbattle&op=occupy&id={id}&region={region_id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    if data.result != "0" {
        return false;
    }

    time::sleep(Duration::from_millis(200)).await;
    data.msg.contains("成功占领资源点")
}

async fn 放弃(d: &DaLeDou) {
    // 放弃
    let data: Response = match d.get("cmd=tbattle&op=abandon").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

#[derive(Deserialize)]
struct Exchange {
    result: String,
    msg: String,
    values: Values,
    items: Vec<Items>, // 兑换材料
}

#[derive(Deserialize)]
struct Values {
    #[serde(rename = "14")]
    score: String, // 积分
}

#[derive(Deserialize)]
struct Items {
    #[serde(rename = "type")]
    t: String,
    name: String,      // 材料名称
    needs_num: String, // 兑换所需积分
}

async fn 积分兑换(d: &DaLeDou) {
    if !d.config().问鼎天下.兑换碎片 {
        return;
    }

    // 积分兑换
    let data: Exchange = match d.get("cmd=exchange&subtype=19").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    let Some(ancient_god_data) = 古阵篇(d).await else {
        return;
    };

    let score: u32 = match data.values.score.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 values 字段失败：{e}"));
            return;
        }
    };

    // 按商店顺序兑换碎片
    for plan in 兑换计划(data.items, &ancient_god_data.treasure_info, score) {
        兑换(d, &plan.t, &plan.name, plan.num as u8).await;
    }
}

#[derive(Deserialize)]
struct AncientGod {
    result: String,
    msg: String,
    #[serde(rename = "treasureInfo")]
    treasure_info: Vec<TreasureInfo>,
}

#[derive(Deserialize)]
struct TreasureInfo {
    level: String, // 当前等级
    #[serde(rename = "sliceName")]
    slice_name: String, // 碎片名称
    #[serde(rename = "sliceConsume")]
    slice_consume: String, // 碎片消耗数量
    #[serde(rename = "sliceHave")]
    slice_have: String, // 碎片拥有数量
    #[serde(rename = "qualityDesc")]
    quality_desc: String, // 配置描述，比如：橙色品质最高提升至12级
}

async fn 古阵篇(d: &DaLeDou) -> Option<AncientGod> {
    // 古阵篇
    let data: AncientGod = match d.get("cmd=ancient_god&op=2").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

/// 单条兑换计划
struct ExchangePlan {
    t: String,    // 物品类型
    name: String, // 碎片名称
    num: u32,     // 兑换数量
}

/// 按商店物品顺序生成兑换计划：只换有缺口的碎片，每种至多 10 个且不超积分
fn 兑换计划(items: Vec<Items>, treasures: &[TreasureInfo], score: u32) -> Vec<ExchangePlan> {
    let mut score = score;
    let mut plan = Vec::new();
    for item in items {
        // 碎片物品按名称匹配宝物
        let Some(treasure) = treasures.iter().find(|t| t.slice_name == item.name) else {
            continue;
        };
        let Some(gap) = 需要碎片(treasure) else {
            continue;
        };
        let Ok(price) = item.needs_num.parse::<u32>() else {
            continue;
        };
        if price == 0 {
            continue;
        }
        let num = cmp::min(gap, cmp::min(10, score / price));
        if num == 0 {
            continue;
        }
        score -= num * price;
        plan.push(ExchangePlan {
            t: item.t,
            name: item.name,
            num,
        });
    }
    plan
}

/// 计算宝物升级所需碎片缺口；品质封顶、碎片已足够或字段解析失败时返回 None
fn 需要碎片(t: &TreasureInfo) -> Option<u32> {
    let max = 最高等级(&t.quality_desc)?;
    let level: u32 = t.level.parse().ok()?;
    if level >= max {
        return None;
    }
    let consume: u32 = t.slice_consume.parse().ok()?;
    let have: u32 = t.slice_have.parse().ok()?;
    let gap = consume.saturating_sub(have);
    (gap > 0).then_some(gap)
}

/// 从品质描述解析最高等级，如「蓝色品质最高提升至6级」→ 6
fn 最高等级(desc: &str) -> Option<u32> {
    let suffix = desc.rsplit_once("至")?.1;
    let digits: String = suffix.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

async fn 兑换(d: &DaLeDou, t: &str, name: &str, num: u8) {
    // 兑换
    let cmd = format!("cmd=exchange&subtype=2&type={t}&times={num}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result == "0" {
        d.log(TASK, &data.msg);
    } else {
        d.log(TASK, &format!("{name}*{num} => {}", data.msg));
    }

    time::sleep(Duration::from_millis(200)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paid_all_times() {
        // 付费：全部使用
        assert_eq!(calc_attack_count(5, true), Some(4));
        assert_eq!(calc_attack_count(1, true), Some(0));
    }

    #[test]
    fn test_free_full() {
        // 免费 5 次 → 至多 3 次 → 2 次 1 级
        assert_eq!(calc_attack_count(5, false), Some(2));
    }

    #[test]
    fn test_free_partial() {
        // 免费 4 次 → 2 次 → 1 次 1 级
        assert_eq!(calc_attack_count(4, false), Some(1));
        // 免费 3 次 → 1 次 → 仅 3 级
        assert_eq!(calc_attack_count(3, false), Some(0));
    }

    #[test]
    fn test_free_no_times() {
        // 免费 2 次以下 → 无
        assert_eq!(calc_attack_count(2, false), None);
        assert_eq!(calc_attack_count(1, false), None);
        assert_eq!(calc_attack_count(0, false), None);
    }

    #[test]
    fn test_paid_no_times() {
        assert_eq!(calc_attack_count(0, true), None);
    }

    // ─── 古阵篇兑换逻辑测试 ───

    fn treasure(
        level: &str,
        slice_name: &str,
        desc: &str,
        consume: &str,
        have: &str,
    ) -> TreasureInfo {
        TreasureInfo {
            level: level.to_string(),
            slice_name: slice_name.to_string(),
            slice_consume: consume.to_string(),
            slice_have: have.to_string(),
            quality_desc: desc.to_string(),
        }
    }

    fn item(t: &str, name: &str, needs_num: &str) -> Items {
        Items {
            t: t.to_string(),
            name: name.to_string(),
            needs_num: needs_num.to_string(),
        }
    }

    // 商店样例：突破石、神魔残卷、四种碎片
    fn sample_items() -> Vec<Items> {
        vec![
            item("1266", "突破石", "40"),
            item("1267", "神魔残卷", "40"),
            item("1268", "烛龙碎片", "40"),
            item("1269", "黄鸟碎片", "40"),
            item("1270", "夔牛碎片", "40"),
            item("1271", "饕餮碎片", "40"),
        ]
    }

    #[test]
    fn test_最高等级() {
        assert_eq!(最高等级("蓝色品质最高提升至6级"), Some(6));
        assert_eq!(最高等级("绿色品质最高提升至3级"), Some(3));
        assert_eq!(最高等级("橙色品质最高提升至12级"), Some(12));
        assert_eq!(最高等级("未知描述"), None);
        assert_eq!(最高等级(""), None);
    }

    #[test]
    fn test_需要碎片_品质封顶不换() {
        // 样例数据：夔牛鼓 蓝色6级满、饕餮鼎 绿色3级满
        assert_eq!(
            需要碎片(&treasure(
                "6",
                "夔牛碎片",
                "蓝色品质最高提升至6级",
                "100",
                "73"
            )),
            None
        );
        assert_eq!(
            需要碎片(&treasure(
                "3",
                "饕餮碎片",
                "绿色品质最高提升至3级",
                "15",
                "158"
            )),
            None
        );
    }

    #[test]
    fn test_需要碎片_缺口() {
        assert_eq!(
            需要碎片(&treasure(
                "2",
                "测试碎片",
                "绿色品质最高提升至3级",
                "15",
                "10"
            )),
            Some(5)
        );
    }

    #[test]
    fn test_需要碎片_碎片已足够() {
        assert_eq!(
            需要碎片(&treasure(
                "2",
                "测试碎片",
                "绿色品质最高提升至3级",
                "15",
                "20"
            )),
            None
        );
    }

    #[test]
    fn test_需要碎片_字段异常() {
        assert_eq!(
            需要碎片(&treasure(
                "x",
                "测试碎片",
                "绿色品质最高提升至3级",
                "15",
                "10"
            )),
            None
        );
        assert_eq!(
            需要碎片(&treasure("2", "测试碎片", "品质描述异常", "15", "10")),
            None
        );
    }

    // 古阵篇样例：四宝均在品质上限
    fn sample_treasures() -> Vec<TreasureInfo> {
        vec![
            treasure("6", "夔牛碎片", "蓝色品质最高提升至6级", "100", "73"),
            treasure("3", "饕餮碎片", "绿色品质最高提升至3级", "15", "158"),
            treasure("3", "烛龙碎片", "绿色品质最高提升至3级", "15", "127"),
            treasure("3", "黄鸟碎片", "绿色品质最高提升至3级", "15", "178"),
        ]
    }

    #[test]
    fn test_兑换计划_样例四宝封顶不换() {
        // 样例：四宝均在品质上限 → 计划为空
        assert!(兑换计划(sample_items(), &sample_treasures(), 17760).is_empty());
    }

    #[test]
    fn test_兑换计划_缺口小于10() {
        // 烛龙印 2 级差 5 个
        let treasures = vec![treasure(
            "2",
            "烛龙碎片",
            "绿色品质最高提升至3级",
            "15",
            "10",
        )];
        let plan = 兑换计划(sample_items(), &treasures, 17760);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].t, "1268");
        assert_eq!(plan[0].num, 5);
    }

    #[test]
    fn test_兑换计划_每天限10() {
        // 夔牛鼓 5 级差 27 个 → 每天限 10，突破石、神魔残卷无匹配宝物被跳过
        let treasures = vec![treasure(
            "5",
            "夔牛碎片",
            "蓝色品质最高提升至6级",
            "100",
            "73",
        )];
        let plan = 兑换计划(sample_items(), &treasures, 17760);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].t, "1270");
        assert_eq!(plan[0].name, "夔牛碎片");
        assert_eq!(plan[0].num, 10);
    }

    #[test]
    fn test_兑换计划_积分限制() {
        // 560 分：烛龙 10 个（400 分）+ 黄鸟 4 个（160 分）
        let treasures = vec![
            treasure("2", "烛龙碎片", "绿色品质最高提升至3级", "15", "0"),
            treasure("2", "黄鸟碎片", "绿色品质最高提升至3级", "15", "0"),
        ];
        let plan = 兑换计划(sample_items(), &treasures, 560);
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].name, "烛龙碎片");
        assert_eq!(plan[0].num, 10);
        assert_eq!(plan[1].name, "黄鸟碎片");
        assert_eq!(plan[1].num, 4);
    }

    #[test]
    fn test_兑换计划_积分不足() {
        let treasures = vec![treasure(
            "2",
            "烛龙碎片",
            "绿色品质最高提升至3级",
            "15",
            "0",
        )];
        assert!(兑换计划(sample_items(), &treasures, 39).is_empty());
    }

    #[test]
    fn test_兑换计划_单价异常跳过() {
        let items = vec![
            item("1268", "烛龙碎片", "x"),
            item("1270", "夔牛碎片", "0"),
            item("1271", "饕餮碎片", "40"),
        ];
        let treasures = vec![
            treasure("2", "烛龙碎片", "绿色品质最高提升至3级", "15", "12"),
            treasure("2", "夔牛碎片", "蓝色品质最高提升至6级", "15", "12"),
            treasure("2", "饕餮碎片", "绿色品质最高提升至3级", "15", "12"),
        ];
        let plan = 兑换计划(items, &treasures, 17760);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].name, "饕餮碎片");
        assert_eq!(plan[0].num, 3);
    }
}
