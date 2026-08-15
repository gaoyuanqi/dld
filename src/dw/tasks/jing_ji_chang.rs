//! 竞技场
//!
//! 免费挑战、领取每日奖励、领取排名奖励、赛季期兑换10个河图洛书

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "竞技场";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    left_free_times: String, // 剩余免费挑战次数
    #[serde(default)]
    can_draw_daily_reward: String, // 是否可以领取每日奖励
    #[serde(default)]
    can_draw_rank_reward: String, // 是否可以领取排名奖励
    #[serde(default)]
    next_season_time: String, // 下赛季开始时间
}

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    // 休赛期
    if data.can_draw_rank_reward == "1" {
        领取排名奖励(d).await;
        return;
    }

    // 赛季进行中
    if data.next_season_time != "-1" {
        return;
    }

    let free_times: u8 = match data.left_free_times.parse() {
        Ok(n) => n,
        Err(e) => {
            d.log(TASK, &format!("解析 left_free_times 失败：{e}"));
            return;
        }
    };

    if free_times > 0 {
        免费挑战(d, free_times).await;
    }

    let Some(data) = query(d).await else {
        return;
    };

    if data.can_draw_daily_reward == "1" {
        领取每日奖励(d).await;
        兑换河图洛书(d).await;
    }
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let data: Query = match d.get("cmd=arena").await {
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

async fn 领取排名奖励(d: &DaLeDou) {
    // 领取排名奖励
    let data: Response = match d.get("cmd=arena&op=rankingreward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 免费挑战(d: &DaLeDou, free_times: u8) {
    #[derive(Deserialize)]
    struct Challenge {
        result: String,
        msg: String,
        #[serde(default)]
        repid: String,
    }

    for _ in 0..free_times {
        // 开始挑战
        let data: Challenge = match d.get("cmd=arena&op=challenge").await {
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

        if !data.repid.is_empty() {
            挑战记录(d, &data.repid).await;
        }
    }
}

async fn 挑战记录(d: &DaLeDou, repid: &str) {
    #[derive(Deserialize)]
    struct QueryRecord {
        result: String,
        msg: String,
        records: Vec<Records>,
    }

    #[derive(Deserialize)]
    struct Records {
        repid: String,
        desc: String,
    }

    // 挑战记录
    let data: QueryRecord = match d.get("cmd=arena&op=queryrecord").await {
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

    for item in &data.records {
        if item.repid == repid {
            d.log(TASK, &item.desc);
            return;
        }
    }
}

async fn 领取每日奖励(d: &DaLeDou) {
    // 领取每日奖励
    let data: Response = match d.get("cmd=arena&op=dailyreward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 兑换河图洛书(d: &DaLeDou) {
    if !d.config().竞技场.兑换河图洛书 {
        return;
    }

    // 兑换10个
    let data: Response = match d.get("cmd=arena&op=exchange&id=5435&times=10").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
