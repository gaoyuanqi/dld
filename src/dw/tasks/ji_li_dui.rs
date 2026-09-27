//! 吉利兑
//!
//! 领取修炼任务和每日任务
//!
//! 活动结束前一天兑换

use std::time::Duration;

use chrono::{Datelike, Local, NaiveDate};
use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "吉利兑";

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in &data.task {
        if item.status == "1" {
            领取(d, &item.taskid).await;
        }
    }

    // 仅活动结束前一天兑换
    let now = Local::now();
    match 是否结束前一天(&data.acttime, now.year(), now.month(), now.day()) {
        Some(true) => {}
        Some(false) => return,
        None => {
            d.log(TASK, &format!("解析活动时间失败：{}", data.acttime));
            return;
        }
    }

    let Some(data) = query(d).await else {
        return;
    };

    // 剩余精魂不足
    if data.myspirit == "0" {
        return;
    }

    let myspirit: u32 = match data.myspirit.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 myspirit 字段失败：{e}"));
            return;
        }
    };

    兑换(d, myspirit, &data.reward).await;
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    myspirit: String, // 剩余精魂
    #[serde(default)]
    acttime: String, // 活动时间
    #[serde(default)]
    task: Vec<Task>,
    #[serde(default)]
    reward: Vec<Reward>,
}

#[derive(Deserialize)]
struct Task {
    taskid: String,
    status: String,
}

#[derive(Deserialize)]
struct Reward {
    rewardid: String,
    goodsname: String, // 兑换物品名称
    hadex: String,     // 已兑换次数
    canex: String,     // 兑换上限
    spirit: String,    // 兑换一次所需精魂
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let data: Query = match d.get("cmd=geelyexchange").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    // 不在活动时间
    if data.result == "-1" {
        return None;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

async fn 领取(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=geelyexchange&op=GetTaskReward&taskid={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

/// 解析活动结束日期（月、日）
///
/// acttime 格式：「9月24日更新后至10月23日6点」，取「至」之后的结束月日
fn 解析结束日期(acttime: &str) -> Option<(u32, u32)> {
    let end = acttime.split_once('至')?.1;
    let (month, rest) = end.split_once('月')?;
    let month: u32 = month.parse().ok()?;
    let (day, _) = rest.split_once('日')?;
    let day: u32 = day.parse().ok()?;
    Some((month, day))
}

/// 判断今天是否为活动结束前一天
///
/// 返回 None 表示 acttime 解析失败
fn 是否结束前一天(acttime: &str, year: i32, month: u32, day: u32) -> Option<bool> {
    let (end_month, end_day) = 解析结束日期(acttime)?;
    // 结束月日早于今天，说明活动跨年，结束在明年
    let end_year = if (end_month, end_day) < (month, day) {
        year + 1
    } else {
        year
    };
    let end = NaiveDate::from_ymd_opt(end_year, end_month, end_day)?;
    let prev = end.pred_opt()?;
    Some(prev.month() == month && prev.day() == day)
}

/// 按配置的兑换优先级依次兑换，每件换到服务器上限，精魂不足跳过
async fn 兑换(d: &DaLeDou, mut myspirit: u32, reward: &[Reward]) {
    let config = &d.config().吉利兑.兑换优先级;
    if config.is_empty() {
        return;
    }

    for want in config {
        let Some(item) = reward.iter().find(|r| want == &r.goodsname) else {
            d.log(TASK, &format!("{want} => 兑换物品不存在"));
            continue;
        };

        if item.spirit == "0" {
            continue;
        }

        let spirit: u32 = match item.spirit.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(
                    TASK,
                    &format!("解析 {} spirit 字段失败：{e}", item.goodsname),
                );
                continue;
            }
        };

        let canex: u32 = match item.canex.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(
                    TASK,
                    &format!("解析 {} canex 字段失败：{e}", item.goodsname),
                );
                continue;
            }
        };
        let hadex: u32 = match item.hadex.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(
                    TASK,
                    &format!("解析 {} hadex 字段失败：{e}", item.goodsname),
                );
                continue;
            }
        };

        let n = 计算兑换数量(canex, hadex, myspirit, spirit);
        if n == 0 {
            continue;
        }
        myspirit -= spirit * n;

        if !兑换一次(d, n, &item.rewardid).await {
            return;
        }
    }
}

/// 计算单个物品本次兑换数量
///
/// `canex` 兑换上限，`hadex` 已兑换次数，`myspirit` 剩余精魂，`spirit` 兑换一次所需精魂
fn 计算兑换数量(canex: u32, hadex: u32, myspirit: u32, spirit: u32) -> u32 {
    if spirit == 0 {
        return 0;
    }
    canex.saturating_sub(hadex).min(myspirit / spirit)
}

/// 兑换一次，返回是否成功
async fn 兑换一次(d: &DaLeDou, n: u32, id: &str) -> bool {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 兑换
    let cmd = format!("cmd=geelyexchange&op=ExchangeProps&rewardid={id}");
    for _ in 0..n {
        let data: Response = match d.get(&cmd).await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return false;
            }
        };

        d.log(TASK, &data.msg);
        if !data.msg.starts_with("精魂兑换成功") {
            return false;
        }

        time::sleep(Duration::from_millis(200)).await;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── 解析结束日期 测试 ───

    // 常规格式解析出结束月日
    #[test]
    fn test_parse_end_date_basic() {
        assert_eq!(解析结束日期("9月24日更新后至10月23日6点"), Some((10, 23)));
    }

    // 跨年活动结束日期
    #[test]
    fn test_parse_end_date_cross_year() {
        assert_eq!(解析结束日期("12月20日更新后至1月15日6点"), Some((1, 15)));
    }

    // 无法解析返回 None
    #[test]
    fn test_parse_end_date_invalid() {
        assert_eq!(解析结束日期(""), None);
        assert_eq!(解析结束日期("abc"), None);
        assert_eq!(解析结束日期("9月24日更新后至10月23"), None);
        assert_eq!(解析结束日期("9月24日更新后至X月23日6点"), None);
    }

    // ─── 是否结束前一天 测试 ───

    // 结束日前一天返回 true
    #[test]
    fn test_end_prev_day_true() {
        assert_eq!(
            是否结束前一天("9月24日更新后至10月23日6点", 2026, 10, 22),
            Some(true)
        );
    }

    // 结束日当天返回 false
    #[test]
    fn test_end_prev_day_end_day() {
        assert_eq!(
            是否结束前一天("9月24日更新后至10月23日6点", 2026, 10, 23),
            Some(false)
        );
    }

    // 结束日之前返回 false
    #[test]
    fn test_end_prev_day_early() {
        assert_eq!(
            是否结束前一天("9月24日更新后至10月23日6点", 2026, 10, 21),
            Some(false)
        );
    }

    // 跨年活动：结束日前一天返回 true
    #[test]
    fn test_end_prev_day_cross_year_true() {
        assert_eq!(
            是否结束前一天("12月20日更新后至1月15日6点", 2026, 1, 14),
            Some(true)
        );
    }

    // 跨年活动：年前日期返回 false
    #[test]
    fn test_end_prev_day_cross_year_false() {
        assert_eq!(
            是否结束前一天("12月20日更新后至1月15日6点", 2026, 12, 31),
            Some(false)
        );
    }

    // 结束日 1月1日：12月31日返回 true
    #[test]
    fn test_end_prev_day_jan_first() {
        assert_eq!(
            是否结束前一天("12月1日更新后至1月1日6点", 2026, 12, 31),
            Some(true)
        );
    }

    // 闰年 2月29日返回 true
    #[test]
    fn test_end_prev_day_leap_year() {
        assert_eq!(
            是否结束前一天("2月10日更新后至3月1日6点", 2024, 2, 29),
            Some(true)
        );
    }

    // 平年 2月28日返回 true
    #[test]
    fn test_end_prev_day_non_leap_year() {
        assert_eq!(
            是否结束前一天("2月10日更新后至3月1日6点", 2025, 2, 28),
            Some(true)
        );
    }

    // 无效日期返回 None
    #[test]
    fn test_end_prev_day_invalid_date() {
        assert_eq!(是否结束前一天("9月1日更新后至2月30日6点", 2026, 1, 1), None);
    }

    // acttime 无法解析返回 None
    #[test]
    fn test_end_prev_day_unparsable() {
        assert_eq!(是否结束前一天("", 2026, 10, 22), None);
    }

    // ─── 计算兑换数量 测试 ───

    // 按剩余上限兑换
    #[test]
    fn test_calc_exchange_max_by_canex() {
        assert_eq!(计算兑换数量(100, 20, 500, 2), 80);
    }

    // 精魂受限
    #[test]
    fn test_calc_exchange_limited_by_spirit() {
        assert_eq!(计算兑换数量(100, 0, 150, 20), 7);
    }

    // 精魂不足
    #[test]
    fn test_calc_exchange_insufficient_spirit() {
        assert_eq!(计算兑换数量(100, 0, 10, 20), 0);
    }

    // 已换满
    #[test]
    fn test_calc_exchange_already_full() {
        assert_eq!(计算兑换数量(25, 25, 1000, 2), 0);
    }

    // 已兑换超过上限的脏数据
    #[test]
    fn test_calc_exchange_hadex_over_canex() {
        assert_eq!(计算兑换数量(25, 30, 1000, 2), 0);
    }

    // 单价为 0
    #[test]
    fn test_calc_exchange_zero_cost() {
        assert_eq!(计算兑换数量(100, 0, 1000, 0), 0);
    }
}
