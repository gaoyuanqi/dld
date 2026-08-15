//! 问鼎天下
//!
//! 领取奖励、攻占、助威
//!
//! 助威：傲龙づ族℃
//!
//! 攻占：
//! 每天总次数 5（3 次免费 + 2 次付费），
//! 积分最大化：打 4 次 1 级失败 + 1 次 3 级成功 = 30×4 + 150 = 270
//! left_fight_times 不区分免费/付费，付费次数固定为 2
//! 开启付费攻占 → left_times 全部使用，1 级 left_times-1 次 + 3 级空资源点 1 次
//! 关闭付费攻占 → 免费次数 = left_times - 2，至多 3 次
//!   left_times=5 → 3 次（1 级 2 次 + 3 级 1 次）
//!   left_times=4 → 2 次（1 级 1 次 + 3 级 1 次）
//!   left_times=3 → 1 次（仅 3 级 1 次）
//!   left_times≤2 → 不打
//! 如果成功攻占一级资源点则放弃

use std::cmp;

use serde::Deserialize;

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
        }
        return;
    }
    if data.duration == "2" {
        if data.cb.can_cheer == "1" && data.cb.cheered_faction_id == "0" {
            排名赛助威(d).await;
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
        let Some(level_1_data) = data.level_1_territories.first() else {
            return;
        };

        for _ in 0..level_1_count {
            if !攻占(d, &level_1_data.canoncial_id, region_id).await {
                continue;
            }

            放弃(d).await;
        }
    }

    let Some(level_3_data) = data.level_3_territories.last() else {
        return;
    };

    // 不是空资源点
    if level_3_data.owner_combat_points != "0.0" {
        return;
    }

    攻占(d, &level_3_data.canoncial_id, region_id).await;
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
}
