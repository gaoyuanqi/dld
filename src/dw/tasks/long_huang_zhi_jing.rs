//! 龙凰之境
//!
//! 报名龙渊赛区、挑战、领奖、兑换
//!
//! 挑战：如果有剩余挑战次数且在战斗期则挑战一次最后一位
//!
//! 每日奖励：每次挑战后领取
//!
//! 排行奖励：休赛期领取
//!
//! 兑换：休赛期兑换

use std::time::Duration;

use chrono::{Datelike, Local, Timelike};
use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "龙凰之境";

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        signin: String, // 报名阵营
        #[serde(default)]
        times: String, // 剩余挑战次数
        #[serde(default, rename = "battleList")]
        battle_list: Vec<BattleList>, // 战斗列表
    }

    #[derive(Deserialize)]
    struct BattleList {
        uin: String, // QQ
    }

    let data: Query = match d.get("cmd=dragon&op=martial").await {
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

    let now = Local::now();
    let day = now.day();

    // 未报名
    if data.signin == "0" {
        // 报名期
        if (1..=3).contains(&day) {
            报名(d).await;
        }
        return;
    }

    if (4..=25).contains(&day) {
        let hour = now.hour();
        // 有剩余挑战次数且在挑战时间
        if data.times != "0" && (8..22).contains(&hour) {
            let Some(battle_data) = data.battle_list.last() else {
                return;
            };
            挑战(d, &data.signin, &battle_data.uin).await;
            每日奖励(d).await;
        }
        return;
    }

    if day < 27 {
        return;
    }

    排行奖励(d).await;
    领奖(d).await;
    兑换(d).await;
}

/// 根据论武次数和位图找出可领取的奖励索引（1~5）
/// 每档阈值：[1, 8, 18, 26, 36]，位图位为 0 表示未领取
fn find_claimable_rewards(times: u32, flag: u32) -> Vec<u32> {
    [1, 8, 18, 26, 36]
        .iter()
        .enumerate()
        .filter(|&(i, &t)| times >= t && (flag >> i) & 1 == 0)
        .map(|(i, _)| i as u32 + 1)
        .collect()
}

async fn 报名(d: &DaLeDou) {
    // 报名龙渊赛区
    let data: Response = match d.get("cmd=dragon&op=martialsign&id=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 挑战(d: &DaLeDou, signin: &str, uin: &str) {
    // 挑战
    let cmd = format!("cmd=dragon&op=challenge&zone={signin}&uin={uin}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 每日奖励(d: &DaLeDou) {
    // 每日奖励
    let data: Response = match d.get("cmd=dragon&op=getdailyprice").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 排行奖励(d: &DaLeDou) {
    // 排行奖励
    let data: Response = match d.get("cmd=dragon&op=getrankprice").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

#[derive(Deserialize)]
struct YunJi {
    result: String,
    msg: String,
    score: String, // 龙凰点
    times: String, // 论武次数
    flag: String,  // 位图值
    #[serde(rename = "shopList")]
    shop_list: Vec<ShopList>, // 商店列表
}

#[derive(Deserialize)]
struct ShopList {
    name: String,
    id: String,
    cost: String,   // 消耗龙凰点
    limit: String,  // 兑换上限
    remain: String, // 剩余兑换数量
}

async fn 龙凰云集(d: &DaLeDou) -> Option<YunJi> {
    // 龙凰云集
    let data: YunJi = match d.get("cmd=dragon&op=yunji").await {
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

async fn 领奖(d: &DaLeDou) {
    let Some(data) = 龙凰云集(d).await else {
        return;
    };

    let times: u32 = match data.times.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 times 字段失败：{e}"));
            return;
        }
    };
    let flag: u32 = match data.flag.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 flag 字段失败：{e}"));
            return;
        }
    };

    for idx in find_claimable_rewards(times, flag) {
        // 领奖
        let cmd = format!("cmd=dragon&op=reward&idx={idx}");
        let data: Response = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
    }
}

async fn 兑换(d: &DaLeDou) {
    let Some(data) = 龙凰云集(d).await else {
        return;
    };

    let exchange = &d.config().龙凰之境.兑换上限;

    // 所有物品配置数量都为 0，无需兑换
    if exchange.凰髓 == 0 && exchange.凰火 == 0 && exchange.龙玉 == 0 && exchange.论武券 == 0
    {
        return;
    }

    let mut score: u32 = match data.score.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 score 字段失败：{e}"));
            return;
        }
    };

    if score < 100 {
        return;
    }

    for shop in &data.shop_list {
        let want = match shop.name.as_str() {
            "凰髓" => exchange.凰髓,
            "凰火" => exchange.凰火,
            "龙玉" => exchange.龙玉,
            "论武券" => exchange.论武券,
            _ => continue,
        };
        if want == 0 {
            continue;
        }

        if shop.remain == "0" {
            continue;
        }

        let remain: u32 = match shop.remain.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} remain 字段失败：{e}", shop.name));
                continue;
            }
        };

        let cost: u32 = match shop.cost.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} cost 字段失败：{e}", shop.name));
                continue;
            }
        };

        if cost == 0 {
            d.log(TASK, &format!("{} 单价为：{cost}", shop.name));
            continue;
        }

        let limit: u32 = match shop.limit.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} limit 字段失败：{e}", shop.name));
                continue;
            }
        };

        let max = calc_max_exchange(want, limit, remain, score, cost);
        if max == 0 {
            continue;
        }

        let (tens, ones) = (max / 10, max % 10);
        score -= cost * max;

        兑换批次(d, &shop.name, &shop.id, tens, 10).await;
        兑换批次(d, &shop.name, &shop.id, ones, 1).await;
    }
}

/// 计算本次可兑换数量
///
/// `want` 用户配置的兑换上限，`limit` 服务器库存总量，
/// `remain` 服务器剩余库存，`score` 当前龙凰点，`cost` 单价
fn calc_max_exchange(want: u32, limit: u32, remain: u32, score: u32, cost: u32) -> u32 {
    // 已兑换 = 上限 - 剩余，还需兑换 = 配置上限 - 已兑换
    let exchanged = limit.saturating_sub(remain);
    let need = want.min(limit).saturating_sub(exchanged);
    need.min(score / cost)
}

async fn 兑换批次(d: &DaLeDou, name: &str, id: &str, count: u32, num: u8) {
    for _ in 0..count {
        let cmd = format!("cmd=dragon&op=shopbuy&id={id}&num={num}");
        let data: Response = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };
        d.log(TASK, &format!("{name}*{num} => {}", data.msg));
        time::sleep(Duration::from_millis(200)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rewards_claimable() {
        // 论武 36 次 + 位图全 0 → 5 档全可领
        assert_eq!(find_claimable_rewards(36, 0), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_partial_times() {
        // 论武 18 次 → 前三档达标
        assert_eq!(find_claimable_rewards(18, 0), vec![1, 2, 3]);
    }

    #[test]
    fn test_times_below_minimum() {
        assert!(find_claimable_rewards(0, 0).is_empty());
    }

    #[test]
    fn test_all_already_claimed() {
        // 位图全 1 → 全已领取
        assert!(find_claimable_rewards(36, 0b11111).is_empty());
    }

    #[test]
    fn test_mixed_state() {
        // 论武 36 次，第 1、3 档已领（位图 bit0=1, bit2=1）
        // flag = 0b00101 = 5
        assert_eq!(find_claimable_rewards(36, 5), vec![2, 4, 5]);
    }

    // ─── calc_max_exchange 测试 ───

    #[test]
    fn test_first_exchange_all_available() {
        // 首次兑换：库存全满，用户配置 10，分数充足
        // want=10, limit=100, remain=100, score=10000, cost=100
        assert_eq!(calc_max_exchange(10, 100, 100, 10000, 100), 10);
    }

    #[test]
    fn test_already_exchanged_enough() {
        // 已兑换数量 >= 配置上限：不再兑换
        // want=10, limit=100, remain=85 → exchanged=15 ≥ want
        assert_eq!(calc_max_exchange(10, 100, 85, 10000, 100), 0);
    }

    #[test]
    fn test_want_less_than_limit() {
        // 配置上限 < 服务器上限，按配置来
        // want=10, limit=100, remain=95 → exchanged=5, need=5
        assert_eq!(calc_max_exchange(10, 100, 95, 10000, 100), 5);
    }

    #[test]
    fn test_want_greater_than_limit() {
        // 配置上限 > 服务器上限，受服务器限制
        // want=100, limit=50, remain=50 → exchanged=0, effective=50
        assert_eq!(calc_max_exchange(100, 50, 50, 10000, 100), 50);
    }

    #[test]
    fn test_score_insufficient() {
        // 分数不够：只能兑能负担的数量
        // want=10, limit=100, remain=100, score=500, cost=100 → max=5
        assert_eq!(calc_max_exchange(10, 100, 100, 500, 100), 5);
    }

    #[test]
    fn test_all_sold_out() {
        // 服务器库存为 0
        // want=10, limit=100, remain=0 → exchanged=100, need=0
        assert_eq!(calc_max_exchange(10, 100, 0, 10000, 100), 0);
    }

    #[test]
    fn test_remain_exceeds_limit() {
        // 数据异常 remain > limit，saturating_sub 兜底
        // want=10, limit=100, remain=110 → exchanged=0, need=10
        assert_eq!(calc_max_exchange(10, 100, 110, 10000, 100), 10);
    }
}
